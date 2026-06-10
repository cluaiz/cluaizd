# `on_path_resolve` Event Hook

The `on_path_resolve` hook fires **exactly once** when the Speculative Graph Search engine successfully finds the shortest valid path to the target neuron.
It allows the DNA script to **react to the outcome of a path search** — most commonly to reinforce successful edges (synaptic plasticity), log telemetry, or trigger downstream side effects.

This is a **post-path** hook. It does not affect which path was chosen — it fires after the winner is already determined.

---

## When It Fires

```
Speculative Search Engine
  │
  ├── Parallel path explorers race to the target
  │     └── First successful path wins
  │
  └── For EVERY neuron on the winning path:
        └── on_path_resolve(ctx) is called
              (executed in order from start → target)
```

> Every neuron on the winning path that has an `on_path_resolve` hook will have it called.
> Neurons NOT on the winning path do NOT have their hook called.

---

## Context Object (`ctx`)

| Property | Type | Description |
| :------- | :--- | :---------- |
| `ctx.neuron.id` | `string` | UUID of the **current** neuron (on the winning path) |
| `ctx.winning_path` | `string[]` | Full ordered list of UUIDs from start → target |
| `ctx.target_id` | `string` | UUID of the destination neuron |
| `ctx.path_length` | `u32` | Number of hops in the winning path |

---

## Available Context Methods

| Method | Description |
| :----- | :---------- |
| `ctx.strengthen_edge(from_id, to_id, delta)` | Increases the weight of an edge by `delta` (0.0–1.0). Weight is clamped to 1.0. |
| `ctx.weaken_edge(from_id, to_id, delta)` | Decreases the weight of an edge by `delta`. Weight is clamped to 0.0. |
| `ctx.log(message)` | Writes a message to the cluaizd telemetry log |

---

## Example 1: Synaptic Reinforcement (Hebbian Learning)

The classic use case — strengthen edges that participated in a successful path:

```rust
// Rhai script: strengthen the edge to the next node on the winning path
fn on_path_resolve(ctx) {
    // Find where we are in the winning path
    let current_idx = -1;
    for i in 0..ctx.winning_path.len() {
        if ctx.winning_path[i] == ctx.neuron.id {
            current_idx = i;
        }
    }

    // Strengthen edge to the next node
    if current_idx >= 0 && current_idx < ctx.winning_path.len() - 1 {
        let next_node = ctx.winning_path[current_idx + 1];
        ctx.strengthen_edge(ctx.neuron.id, next_node, 0.25);
    }
}
```

After this script runs, the weight on the C→D edge increases from `0.5` to `0.75`, making this path more likely to be chosen in future searches.

---

## Example 2: Telemetry Logging

```rust
// Rhai script: log each winning path to the telemetry stream
fn on_path_resolve(ctx) {
    ctx.log(`Path resolved: length=${ctx.path_length}, via=${ctx.neuron.id}`);
}
```

---

## Example 3: Adaptive Decay on Losing Paths

Combine with `on_path_step` to also weaken edges that were explored but did NOT win:

```rust
// This on_path_resolve runs for winners.
// Use a separate on_traverse hook to weaken edges that were pruned.
fn on_path_resolve(ctx) {
    let current_idx = ctx.winning_path.index_of(ctx.neuron.id);
    if current_idx >= 0 && current_idx < ctx.winning_path.len() - 1 {
        ctx.strengthen_edge(ctx.neuron.id, ctx.winning_path[current_idx + 1], 0.1);
    }
}
```

---

## Performance Notes

| Aspect | Detail |
| :----- | :----- |
| **Execution frequency** | Once per neuron on the winning path — very low overhead |
| **Idempotency** | `strengthen_edge` uses atomic LMDB writes — safe to call concurrently |
| **Side effect scope** | Changes to edge weights are durable and visible to all future traversals |

---

## Related

- [`on_path_step`](./on_path_step.md) — Node-level hook during speculative path exploration
- [`on_traverse`](./on_traverse.md) — Edge-level hook during regular graph traversal
- [Speculative Graph Routing](../../architecture/speculative-routing.md) — Full architecture overview
