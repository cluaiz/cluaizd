# `on_traverse` Event Hook

The `on_traverse` hook is an execution affordance triggered **at each hop** during a graph traversal operation.
It allows the DNA script to dynamically control which edges the traversal engine follows — enabling path filtering, weight-based pruning, and context-aware routing at the memory pointer level.

---

## When It Fires

`on_traverse` fires **once per edge** as the graph engine walks the adjacency list of a neuron.
It is called **before** the engine commits to following the edge, giving the script the ability to accept or reject each potential next hop.

```
Traversal Engine
  │
  ├── Reads neuron.adjacency[]
  │     For each edge:
  │     ├── Calls on_traverse(ctx, edge)
  │     │     ├── returns true  → engine follows this edge (hop proceeds)
  │     │     └── returns false → engine prunes this edge (hop skipped)
  │     └── Recurses into accepted edges
  └── Aggregates results
```

---

## Context Object (`ctx`)

| Property | Type | Description |
| :------- | :--- | :---------- |
| `ctx.neuron.id` | `string` | UUID of the **current** neuron being evaluated |
| `ctx.edge.target_id` | `string` | UUID of the **candidate next** neuron |
| `ctx.edge.weight` | `f64` | Weight of this directed edge (0.0–1.0) |
| `ctx.edge.last_accessed_ns` | `u64` | Nanosecond timestamp of last traversal |
| `ctx.depth` | `u32` | Current traversal depth (hops from start) |

---

## Return Value

| Return | Effect |
| :----- | :----- |
| `true` | Engine **follows** this edge — hop proceeds |
| `false` | Engine **prunes** this edge — hop skipped |

---

## Example 1: Weight-Based Pruning (Only Strong Connections)

```rust
// Rhai script: only follow edges with weight >= 0.6
fn on_traverse(ctx) {
    return ctx.edge.weight >= 0.6;
}
```

---

## Example 2: Depth-Limited Traversal

```rust
// Rhai script: stop exploring beyond depth 3
fn on_traverse(ctx) {
    return ctx.depth <= 3;
}
```

---

## Example 3: Recency Filter (Only Recently Used Edges)

```rust
// Rhai script: skip edges not accessed in last 24 hours
// 86_400_000_000_000 ns = 24 hours
fn on_traverse(ctx) {
    let age_ns = ctx.time_now_ns() - ctx.edge.last_accessed_ns;
    return age_ns < 86_400_000_000_000;
}
```

---

## Example 4: Dynamic Routing via Payload Field

```rust
// Rhai script: only follow edges if target's category matches
fn on_traverse(ctx) {
    let target = ctx.fetch_neuron(ctx.edge.target_id);
    let payload = parse_json(target.raw_payload);
    return payload["category"] == "finance";
}
```

---

## Performance Notes

| Aspect | Detail |
| :----- | :----- |
| **Execution frequency** | Called once per edge — avoid heavy computation here |
| **Memory safety** | Script runs in sandboxed Rhai context; no direct memory access |
| **Caching** | Pre-compiled Rhai/WASM scripts are cached — instantiation cost is O(1) |

---

## Related

- [`on_path_step`](./on_path_step.md) — Controls speculative path routing steps
- [`on_path_resolve`](./on_path_resolve.md) — Fires when winning path is found
- [`on_read`](./on_read.md) — Fires on single neuron read operations
