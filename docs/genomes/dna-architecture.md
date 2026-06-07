# DNA Architecture (Genomes)

CLUAIZD relies on a revolutionary concept called **Genomes**. The core rust engine (`engine-lmdb`) contains zero business logic, schemas, or indexing optimizations. It only understands how to rapidly serialize and deserialize byte arrays.

All intelligence is injected into the engine using DNA.

## What is a Genome?
A Genome is an isolated script (written in WASM or Rhai) that attaches to a Neuron. When the CLUAIZD core engine interacts with that Neuron, it triggers specific hooks defined by the Genome.

### The 5 Biological Hooks

Every Genome can implement up to **5 hooks**. Each hook is a Rhai script string (or WASM function) that fires at a specific lifecycle event:

---

#### 1. `on_write` — Ingestion Gate
**Fires:** Before every `POST /neuron` write, before WAL commit.  
**Returns:** `#{ action: "Allow" }` to permit, or `#{ action: "Abort", error: "msg" }` to reject.  
**Use for:** Schema validation, append-only enforcement, field presence checks.

```json
{
  "on_write": "let res = #{ action: \"Allow\" };\nif !payload.contains(\"id\") { res.action = \"Abort\"; res.error = \"Missing id\"; }\nres"
}
```

---

#### 2. `on_read` — Access Control & Transformation
**Fires:** When a CNQL query reads this neuron's payload.  
**Returns:** `#{ payload: modified_payload }`.  
**Use for:** Row-Level Security (decrypt for authorized users), field masking, live data reshaping.

```json
{
  "on_read": "let res = #{ payload: payload };\nif !caller.has_role(\"admin\") { res.payload.remove(\"ssn\"); }\nres"
}
```

---

#### 3. `on_index` — Async Post-Write Indexing
**Fires:** Asynchronously AFTER a successful write (non-blocking).  
**Returns:** `#{ text_fields: ["title", "body"], language: "english" }`.  
**Use for:** Extracting fields into the Inverted Index for full-text search, generating vector embeddings.

```json
{
  "on_index": "let res = #{};\nres.text_fields = [\"title\", \"content\", \"tags\"];\nres.language = \"english\";\nres"
}
```

---

#### 4. `on_lifecycle` — Dreamer Eviction & Tiering
**Fires:** Every N seconds by the background Dreamer engine (configurable in `config.toml`).  
**Returns:** `#{}` for no action, `#{ action: "Evict" }` to delete, `#{ new_tier: "Cold", compress_level: 9 }` to compress.  
**Use for:** TTL eviction (Redis-like), automatic Cold-tier demotion for old data.

```json
{
  "on_lifecycle": "let res = #{};\nif age_ns > config.ttl_ns { res.action = \"Evict\"; }\nres",
  "parameters": { "ttl_ns": 3600000000000 }
}
```

---

#### 5. `on_traverse` — Graph Edge Filtering *(Advanced)*
**Fires:** During `GET /graph/{id}/traverse` — once per edge, deciding whether to follow it.  
**Returns:** `#{ follow: true }` to include the edge, `#{ follow: false }` to skip it.  
**Use for:** Access-controlled graph traversal (skip edges below a weight threshold, skip edges to blocked users).

```json
{
  "on_traverse": "let res = #{ follow: true };\nif edge_weight < config.min_weight { res.follow = false; }\nres",
  "parameters": { "min_weight": 0.5 }
}
```

> [!NOTE]
> `on_traverse` is only evaluated by `GET /graph/{id}/traverse`. The CNQL `traverse()` pipeline operator uses its own `min_weight` parameter instead. Use `on_traverse` for HTTP graph traversal with per-neuron access control policies.

---

## Full Genome Template

```json
{
  "engine": "rhai",
  "on_write": "let res = #{ action: \"Allow\" }; res",
  "on_read": "let res = #{ payload: payload }; res",
  "on_index": null,
  "on_lifecycle": "let res = #{}; if age_ns > config.ttl_ns { res.action = \"Evict\"; } res",
  "on_traverse": null,
  "parameters": {
    "ttl_ns": 3600000000000,
    "min_weight": 0.0
  },
  "wasm_module": null
}
```

## WASM vs Rhai

| Feature | Rhai (`.json`) | WASM (`.wasm`) |
|---|---|---|
| Authoring | Simple JSON-embedded scripts | Compiled C/Rust binary |
| Performance | ~0.1ms per hook | ~0.01ms per hook (near-native) |
| Use case | Schema validation, TTL rules, access control | Vector math, text parsing, heavy compute |
| Hot-reload | ✅ Yes — change JSON file, restart not needed | ⚠️ Requires re-upload via `/booster/upload` |
