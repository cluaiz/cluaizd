# Bits to Atoms (Biological Storage Tiering)

In a traditional database system, encountering an Out-Of-Memory (OOM) error causes the entire application to crash. The database aggressively holds onto caches until the OS kills it. 

CLUAIZD takes a biological approach to memory management through its **Bits to Atoms** architecture, governed by the background *Dreamer* engine.

## The 3 Storage Tiers

Data inside a `UniversalNeuron` is not static. Based on its access frequency and the overall system pressure (RAM availability), it migrates between three tiers:

### 1. Tier 1: Hot (The Conscious Mind)
- **Location:** Pure LMDB memory-mapped access.
- **Latency:** `< 1 ms`.
- **Payload:** Fully uncompressed, raw JSON/Binary payload.
- **Use Case:** Actively queried data, real-time sensory streams, UI state.

### 2. Tier 2: Warm (The Subconscious)
When memory pressure builds, the Dreamer engine evaluates TTL (Time-To-Live) and access frequencies. If a Neuron is deemed less important, it is moved to the Warm tier.
- **Location:** Disk-backed LMDB.
- **Latency:** `1-5 ms`.
- **Payload:** The heavy `raw_payload` is DELETED to save space. However, the **Vector Embeddings** and **Graph Edges** (Adjacency) are retained.
- **Use Case:** The system cannot read the exact payload, but it retains an "intuition" about the data. You can still run Vector Searches and Graph Traversals on Warm neurons, scoring them lower than Hot neurons.

### 3. Tier 3: Cold (Deep Memory)
- **Location:** Compressed file chunks on disk (or Object Store).
- **Latency:** `> 50 ms` (Requires Rehydration).
- **Payload:** The entire Neuron is aggressively compressed using the ZSTD algorithm.
- **Use Case:** Archival logs, heavy Blob media. The data is not lost, but fetching it requires an asynchronous background process to "rehydrate" it back into the Hot tier.

## The Dreamer Engine
The Dreamer is an autonomic background thread. You do not configure crons or run maintenance scripts. 
It monitors `sysinfo` (RAM usage). If RAM drops below 15%, it automatically transitions the oldest Hot neurons to Warm, and Warm to Cold. This ensures CLUAIZD **never OOM crashes**, gracefully degrading performance instead of failing entirely.
