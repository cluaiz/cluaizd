# LMDB & Zero-Copy Architecture

> *"The fastest code is the code that never runs. The fastest copy is the copy that never happens."*

## What is LMDB?

LMDB (Lightning Memory-Mapped Database) is the storage engine at the heart of CLUAIZD. It was originally developed by Howard Chu at Symas Corporation for the OpenLDAP project. Despite being less famous than RocksDB, LMDB holds several world records in database benchmarks:

- **Fastest random read latency:** `< 1 microsecond` on NVMe SSDs.
- **Zero read amplification:** A single `get()` call reads exactly the bytes requested — no extra data.
- **Crash safety without `fsync`:** LMDB uses a copy-on-write B-tree. Even without an explicit disk flush, the database remains consistent after a crash.

---

## B-Tree + Memory-Mapped Files: The Architecture

LMDB stores all data in a single `.mdb` file on disk. This file is accessed via **memory-mapped I/O (`mmap`)**.

### How `mmap` Works
When CLUAIZD opens an LMDB environment, it asks the OS to "map" the entire `.mdb` file into its virtual address space. This means:

```
Physical Disk:  [.mdb file — 4 GB]
                       │ mmap()
                       ▼
Virtual Memory: [0x7f000000 ... 0x8f000000]  ← CLUAIZD process's address space
```

Reading a record is now simply dereferencing a pointer in virtual memory. The OS page cache handles the actual disk I/O transparently. If the page is already cached in RAM (which it usually is for hot data), the read is **entirely in RAM — no disk I/O at all**.

---

## Zero-Copy Deserialization

In most databases, reading a record involves:
1. OS reads bytes from disk into kernel buffer.
2. OS copies bytes from kernel buffer to userspace buffer.
3. Application deserializes bytes into a struct (malloc + memcpy).

CLUAIZD + LMDB eliminates steps 2 and 3 for read operations:

```
Traditional:   Disk → Kernel Buffer → Copy → Userspace Buffer → Deserialize → Struct
CLUAIZD/LMDB:   Disk → OS Page Cache → mmap pointer → Rust slice reference (ZERO COPIES)
```

The Rust code receives a raw `&[u8]` slice pointing directly into the memory-mapped file. No allocation. No copying. This is called **zero-copy deserialization** and it is why CLUAIZD achieves sub-microsecond read latencies.

---

## Copy-On-Write B-Tree: The Crash Safety Guarantee

LMDB uses a **Copy-On-Write (CoW) B-Tree**. When you write a new record:

1. LMDB does NOT modify the existing B-tree pages.
2. It allocates NEW pages, writes the updated data there, then atomically swaps the root pointer.
3. The old pages remain on disk until a vacuum cycle removes them.

```
Before Write:     Root → [Page A] → [Page B] → Data
                         (old)        (old)

During Write:     Root → [Page A'] → [Page B'] → New Data   (atomic swap)
                         (new)         (new)
                  Root → [Page A]  → [Page B]  → Old Data   (still valid until freed)

After Crash:      Root still points to old data — database is consistent!
```

This means LMDB is always consistent, even without `fsync`, because the old tree is never destroyed until the new tree is fully committed.

---

## Why CLUAIZD Chose LMDB Over RocksDB

| Factor | RocksDB | LMDB |
|---|---|---|
| Read Latency | ~50-100µs (LSM compaction overhead) | ~1µs (direct mmap) |
| Write Amplification | High (LSM compaction rewrites data 10-30x) | Low (CoW B-tree) |
| Read Amplification | High (may read multiple SSTables) | 1x (single lookup) |
| Crash Safety | Requires WAL | Built-in (CoW tree) |
| Memory Usage | High (bloom filters, block caches) | Minimal |
| Complexity | Very high | Low |

For CLUAIZD's use case (fast reads for Hot neurons, AI queries, graph traversal), LMDB's sub-microsecond read latency is far more important than RocksDB's slightly better sequential write throughput.

---

## CLUAIZD's WAL on Top of LMDB

LMDB already provides crash safety via CoW. So why does CLUAIZD add its own WAL?

LMDB's CoW only protects data that was **fully committed** before the crash. If the OS crashes while LMDB is in the middle of writing a large transaction, that transaction is silently lost — LMDB just rolls back to the previous consistent state.

CLUAIZD's WAL captures the intent of every operation BEFORE it is submitted to LMDB. On the next boot, CLUAIZD can replay any WAL entries that LMDB lost, ensuring zero data loss even for partially committed transactions.

```
User Write → WAL Append (disk sync) → LMDB Write (async)
           
On Crash Recovery:
  WAL scan → Replay missing entries → LMDB now consistent
```
