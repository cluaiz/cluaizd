# 🧬 Genome Hooks Reference

> *"A database shouldn't just store data; it should protect itself, organize itself, and dream for itself."*

CLUAIZD allows every neuron (row/document) to carry its own "DNA" — a set of logic scripts executed dynamically inside the database engine. This logic is triggered via 4 distinct **Lifecycle Hooks**.

## The 4 Hooks

| Hook | When it Runs | Purpose | Engine Behavior |
|---|---|---|---|
| `on_write` | Right before the neuron is written to LMDB. | Validation, Access Control, Data Sanitization. | If it returns `Reject`, the write fails with HTTP 403. |
| `on_read` | Every time the neuron is queried via `/neuron` or CDQL. | Data masking, logging, auditing, decryption. | Can modify the returned data *without* altering the stored disk data. |
| `on_index` | During write or CDQL indexing phases. | Custom embedding generation, automatic edge creation. | Can extract traits and build graph edges dynamically. |
| `on_lifecycle` | Periodically by the background Dreamer GC. | Autonomic memory management. | Can return `Evict`, `Cold`, or `Allow` to force GC behavior. |
| `on_dream` | Periodically by the background Dreamer Stochastic Walk. | Generative subconscious thought. | Forges new edges randomly during idle CPU time. |

---

## 1. The `on_write` Hook

The `on_write` hook is the gatekeeper. It has access to the incoming `payload` and user context.

### Rhai Example: Strict Schema Validation
```rust
// In DNA genome parameter:
"on_write": "
    if !payload.contains_key(\"email\") {
        return #{ action: \"Reject\", reason: \"Missing email field\" };
    }
    if !payload.email.contains(\"@\") {
        return #{ action: \"Reject\", reason: \"Invalid email format\" };
    }
    #{ action: \"Allow\" }
"
```

---

## 2. The `on_read` Hook

The `on_read` hook intercepts data *before* it is returned to the user. It is perfect for PII masking.

### Rhai Example: PII Masking
```rust
// Mask the social security number and email
"on_read": "
    let masked_payload = payload;
    if masked_payload.contains_key(\"ssn\") {
        masked_payload.ssn = \"***-**-****\";
    }
    if masked_payload.contains_key(\"email\") {
        masked_payload.email = \"[REDACTED]\";
    }
    #{ action: \"Modify\", payload: masked_payload }
"
```

---

## 3. The `on_index` Hook

Use `on_index` to automatically extract tags or create graph edges without the client needing to do it.

### Rhai Example: Auto-Tagging
```rust
"on_index": "
    let mut new_edges = [];
    if payload.contains_key(\"category\") {
        new_edges.push(#{ target: \"tag_\" + payload.category, weight: 1.0 });
    }
    #{ action: \"Allow\", create_edges: new_edges }
"
```

---

## 4. The `on_lifecycle` Hook

This hook communicates directly with the **Dreamer GC**.

### Rhai Example: Ephemeral Data (Self-Destruct)
```rust
"on_lifecycle": "
    // If the neuron is older than the TTL in our parameters
    if current_time_ns - neuron_created_ns > config.ttl_ns {
        return #{ action: \"Evict\" }; // Delete from disk entirely
    }
    #{ action: \"Allow\" }
"
```

### Rhai Example: Auto-Archiving
```rust
"on_lifecycle": "
    if current_time_ns - neuron_created_ns > 86400000000000 { // 1 day
        return #{ action: \"Cold\" }; // Compress with ZSTD and move to Cold storage
    }
    #{ action: \"Allow\" }
"
```

---

## The `on_dream` Hook

Introduced with the Dreaming Mode feature, this hook is executed when the system is idle.

### Rhai Example: Stochastic Ideation
```rust
"on_dream": "
    // Randomly create a link to another neuron if they share a structural trait
    if rand_float() > 0.8 {
        return #{ create_edge: true, target: \"concept_\" + (rand() % total_neurons), weight: 0.2 };
    }
    #{ create_edge: false }
"
```
