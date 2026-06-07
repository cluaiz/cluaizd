# The Dreamer Engine: Autonomic Memory Management

> *"A brain that never forgets crashes. A brain that forgets gracefully survives."*

## Why Every Other Database Eventually Crashes

Traditional databases have one strategy for memory management: **"Keep everything in RAM until you can't, then crash."**

PostgreSQL has `shared_buffers`. Redis has `maxmemory`. InfluxDB has TSM compaction. All of them require manual tuning, and all of them have the same failure mode: when RAM runs out, the OS kills the process.

CNSDB takes a radically different approach inspired by biological nervous systems.

---

## The Biological Inspiration

Human brains handle more information than RAM can hold. The brain's solution:
- **Working Memory (RAM):** What you are actively thinking about RIGHT NOW.
- **Short-Term Memory (Warm):** What happened today. Accessible quickly.
- **Long-Term Memory (Cold):** What happened years ago. Requires effort to recall but never truly lost.

The **Dreamer** is CNSDB's background thread that autonomically manages this three-tier memory hierarchy. It runs every few seconds, evaluating whether neurons need to be promoted, demoted, or evicted.

---

## How the Dreamer Decides What to Demote

The Dreamer uses **real-time system metrics** via the `sysinfo` crate. It monitors:
- Available RAM as a percentage of total RAM.
- Number of Hot-tier neurons currently in memory.
- Each neuron's `last_accessed_at` timestamp (age).
- The `on_lifecycle` hook result from each neuron's genome.

### The Decision Tree

```
Every N seconds, for each Hot Neuron:
  ↓
  Step 1: Call neuron's genome `on_lifecycle` hook
          ├─ Returns "Evict" → Delete immediately (TTL expiry)
          ├─ Returns "Cold" → ZSTD compress, move to Cold tier
          └─ Returns {} (no action) → continue to Step 2
  ↓
  Step 2: Check RAM pressure
          ├─ RAM > 85% used → Demote oldest-accessed Hot neurons to Warm
          ├─ RAM > 95% used → Demote oldest Warm neurons to Cold (ZSTD compress)
          └─ RAM < 85% → Do nothing
```

---

## The 3 Tiers in Detail

### Tier 1: Hot (Conscious Working Memory)
- **Data:** Full `raw_payload` + `vector_data` + `adjacency` in RAM.
- **Latency:** `< 1ms` — LMDB mmap pointer dereference.
- **Queries:** All CNQL operations work at full speed.

### Tier 2: Warm (Subconscious Cache)
- **Data:** `raw_payload` is DELETED from LMDB to save space. Only the neuron shell (ID, `vector_data`, `adjacency`) is retained.
- **Latency:** `1-5ms` — Reading from disk-backed LMDB.
- **Queries:** Vector search and graph traversal still work (using retained vectors and edges). Content queries (`filter name: "Aryan"`) return empty — the payload is gone.
- **Why Warm?** The AI system retains an "intuition" about the data even when the exact content is not immediately available.

### Tier 3: Cold (Deep Long-Term Memory)
- **Data:** Entire neuron (payload + vectors + edges) stored as a ZSTD Level 9 compressed blob.
- **Latency:** `50ms+` — Requires background decompression and LMDB rewrite.
- **Queries:** Cannot be queried directly. Must be "rehydrated" back to Hot first (triggered automatically when queried).

---

## Rehydration: Waking a Cold Neuron

When a query targets a Cold neuron, the Dreamer automatically rehydrates it:

1. CNQL query hits a Cold-tier Neuron ID.
2. Dreamer is notified: "Rehydrate `neuron_xyz`".
3. Dreamer decompresses the ZSTD blob in a background thread.
4. Writes the decompressed data back to LMDB as a Hot neuron.
5. Query is re-executed, now hitting the Hot neuron.

This is transparent to the user — the first query for a Cold neuron is slower (50ms+), subsequent queries hit it as Hot (`<1ms`).

---

## Configuring the Dreamer

In `config.toml`:
```toml
[dreamer]
# How often the Dreamer evaluates neurons (in seconds)
scan_interval_seconds = 10

# Start demoting Hot → Warm when free RAM drops below this %
ram_warn_threshold = 0.20

# Emergency: start compressing Warm → Cold when free RAM drops below this %
ram_critical_threshold = 0.05

# Default cold TTL for neurons with no on_lifecycle hook
default_cold_ttl_seconds = 3600
```

---

## Why This Approach is Superior to Manual Tuning

| Approach | PostgreSQL `shared_buffers` | Redis `maxmemory` | CNSDB Dreamer |
|---|---|---|---|
| Configuration | Manual, requires DBA expertise | Manual, single limit | Automatic, self-calibrating |
| Behavior at RAM limit | Query slowdown, OOM kill | Evict or crash | Graceful Hot→Warm→Cold |
| Data loss at limit | ❌ Never (WAL) | ✅ Possible (eviction) | ❌ Never (data moves to Cold) |
| Tuning required | ✅ Yes, complex | ✅ Yes, tricky | ❌ Optional |
| Biological analogy | Fixed-size workbench | Sticky notes with eraser | Human brain |
