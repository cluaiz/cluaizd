# Key-Value Genome (`ephemeral_cache.json`)

> *"When milliseconds matter, skip everything."*

## When to Use This Genome
Use the `ephemeral_cache` genome when:
- You need sub-millisecond data retrieval with no processing overhead.
- Data has a natural expiry (user sessions, rate-limit counters, presence flags).
- You are building a distributed cache layer.
- You need an atomic counter (e.g., active WebSocket connections).

Real-world use cases: Session tokens, API rate limiting, leaderboard counters, online presence, feature flags, temporary job queues.

---

## The O(1) Fast-Path: How It Actually Works

This is CLUAIZD's most powerful optimization. When the CDQL Planner sees a `find id("...")` query, it **completely skips** all query planning and WASM/Rhai evaluation. Instead, it calls LMDB's direct key lookup which is a single memory-mapped pointer dereference.

```
Normal Query Path:
  CDQL String → Lexer → Parser → Planner → WASM Eval → LMDB Scan → Filter → Result

Fast-Path (Key-Value):
  find id("x") → LMDB Direct Get → Result
```

The difference is `10ms` vs `< 0.1ms` for a typical record fetch.

### Why LMDB is Perfect for Key-Value
LMDB uses a **B-tree backed by memory-mapped files**. When you call `get("key")`, the OS page cache may already have that page in RAM. The kernel maps the physical RAM page directly into your Rust process's virtual address space. No copying. Zero serialization overhead. This is called **zero-copy deserialization**.

---

## The TTL Eviction Lifecycle

The `ephemeral_cache.json` genome's `on_lifecycle` hook is evaluated by the Dreamer background thread on every scan cycle. If the Neuron's age exceeds the TTL, the hook returns `action: "Evict"`.

```json
{
  "on_lifecycle": "let res = #{};\nif age_ns > config.ttl_ns {\n    res.action = \"Evict\";\n}\nres",
  "parameters": { "ttl_ns": 600000000000 },
  "engine": "rhai"
}
```

`ttl_ns: 600000000000` = 10 minutes in nanoseconds.

---

## Storing a Session Token (In-Memory Cache `SET EX` Equivalent)

```bash
curl -X POST http://localhost:7331/data \
  -H "Content-Type: application/json" \
  -d '{
    "id": "session_tok_abc123",
    "tier": "Hot",
    "raw_payload": [bytes for {"user_id": "user_aryan", "role": "admin"}],
    "vector_data": [],
    "adjacency": [],
    "dna": {
      "engine": "rhai",
      "on_lifecycle": "let res = #{}; if age_ns > 1800000000000 { res.action = \"Evict\"; } res",
      "parameters": {}
    }
  }'
```

This session expires in 30 minutes (1,800,000,000,000 nanoseconds).

---

## Retrieving a Session Token (0ms Fast-Path)

```bash
curl -X POST http://localhost:7331/query \
  -H "Content-Type: application/json" \
  -d '{"cdql": "find id(\"session_tok_abc123\")"}'
```

The Planner detects `id(...)` and bypasses the entire WASM engine, returning the result in under `0.1ms`.

---

## Atomic Counters (Rate Limiting)

CLUAIZD does not yet have native atomic increment. The recommended pattern is to use `on_write` to read the current counter, increment it, and enforce the limit:

```json
{
  "on_write": "
    let res = #{ action: 'Allow' };
    if payload.request_count > 1000 {
      res.action = 'Abort';
      res.error = 'Rate limit exceeded (1000 req/hour)';
    }
    res
  "
}
```

---

## Comparison: CLUAIZD vs In-Memory Cache

| Feature | In-Memory Cache | CLUAIZD (ephemeral_cache) |
|---|---|---|
| O(1) Get | ✅ | ✅ (via Fast-Path) |
| TTL Eviction | ✅ | ✅ (via Dreamer lifecycle hook) |
| Data Structures (Lists, Sets) | ✅ | ⚠️ (via JSON payload, not native) |
| Vector Similarity | ❌ | ✅ (switch genome) |
| Graph Edges | ❌ | ✅ (switch genome) |
| Persistence (Crash Recovery) | ⚠️ (RDB/AOF) | ✅ (WAL always on) |
| Horizontal Cluster | ✅ (In-Memory Cache Cluster) | 🔜 (Planned) |
