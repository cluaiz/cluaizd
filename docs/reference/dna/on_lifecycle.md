# `on_lifecycle` Event Hook

The `on_lifecycle` hook is a synchronous event trigger invoked by the background **compactor/dreamer thread** when scanning neurons to manage system resource limits. It enables dynamic garbage collection, database state compaction, and memory tier transition policies.

---

## Hook Signature

### Rhai Engine
In Rhai scripts, the hook must return a `Map` containing lifecycle directives:

```rust
// on_lifecycle script
let age_ns = neuron.age_ns;
let current_tier = neuron.current_tier;

// Example Policy: transition to Warm tier after 60 seconds
if age_ns > 60000000000 {
    return #{
        "new_tier": "Warm",
        "clear_payload": true // Free disk page space by stripping payload
    };
}

return #{};
```

---

## Returned Directives Map

| Key | Type | Description |
| :--- | :--- | :--- |
| **`delete_neuron`** | `bool` | If `true`, triggers immediate **Apoptosis** (physical record deletion). |
| **`new_tier`** | `string` | Transitions storage tier. Supported targets: `"Warm"`, `"Cold"`. |
| **`clear_payload`** | `bool` | If `true`, strips the raw payload to reclaim memory while keeping indices/vectors. |
| **`edge_decay_factor`** | `float` | Multiplies graph adjacency weights (decay rate). |
| **`edge_prune_threshold`** | `float` | Retains graph edges only if weight is equal to or greater than this value. |

---

## Complete Rhai Example: Hybrid Compaction Policy

This script decays edges over time and performs apoptosis if the node becomes disconnected:

```rust
// genomes/compact_rules.rhai
let age_ns = neuron.age_ns;

let result = #{};

// 1. Decay edges by 5% each compaction pass
result.edge_decay_factor = 0.95;
result.edge_prune_threshold = 0.1; // Prune if weight drops below 10%

// 2. Transition payload to Warm tier after 5 minutes
if age_ns > 300000000000 {
    result.new_tier = "Warm";
    result.clear_payload = true;
}

// 3. Trigger Apoptosis if older than 1 hour
if age_ns > 3600000000000 {
    result.delete_neuron = true;
}

return result;
```
