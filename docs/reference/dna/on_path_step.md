# `on_path_step` Event Hook

The `on_path_step` hook fires **at every step** of the **Speculative Graph Search** engine.
It gives the DNA script full control over whether the current neuron should be included in the speculative path being explored — enabling surgical path pruning without modifying the graph structure.

This hook is the primary mechanism for implementing intelligent path routing, such as avoiding compromised nodes, enforcing policy boundaries, or prioritizing high-confidence routes.

---

## When It Fires

`on_path_step` fires when the Speculative Search engine considers including the host neuron in a candidate path:

```
Speculative Search Engine
  │
  ├── Spawns N parallel path explorers
  │     Each explorer, at each node:
  │     ├── Calls on_path_step(ctx)
  │     │     ├── returns true  → node is included; path continues
  │     │     └── returns false → path is PRUNED (this entire branch abandoned)
  │     └── First explorer to reach target wins
  └── Winning path returned to caller
```

---

## Context Object (`ctx`)

| Property | Type | Description |
| :------- | :--- | :---------- |
| `ctx.neuron.id` | `string` | UUID of the current neuron being evaluated |
| `ctx.neuron.raw_payload` | `bytes` | Raw payload of the current neuron |
| `ctx.path_so_far` | `string[]` | Ordered list of neuron UUIDs on the current candidate path |
| `ctx.depth` | `u32` | Current path depth |
| `ctx.target_id` | `string` | UUID of the destination neuron being searched for |

---

## Return Value

| Return | Effect |
| :----- | :----- |
| `true` | Neuron **accepted** — path explorer continues through this node |
| `false` | Neuron **rejected** — entire candidate path is **pruned and abandoned** |

> **Important**: Returning `false` prunes the entire remaining branch from this node, not just this hop. Design pruning logic carefully.

---

## Example 1: Simple Allow-All (Default Behaviour)

```rust
// Rhai script: allow all nodes (no pruning)
fn on_path_step(ctx) {
    return true;
}
```

---

## Example 2: Block Quarantined Nodes

```rust
// Rhai script: prune any node marked as quarantined
fn on_path_step(ctx) {
    let payload = parse_json(ctx.neuron.raw_payload);
    return payload["status"] != "quarantined";
}
```

---

## Example 3: Maximum Path Length Enforcement

```rust
// Rhai script: do not explore paths longer than 5 hops
fn on_path_step(ctx) {
    return ctx.depth <= 5;
}
```

---

## Example 4: Loop Detection

```rust
// Rhai script: prevent revisiting nodes (cycle detection)
fn on_path_step(ctx) {
    return !ctx.path_so_far.contains(ctx.neuron.id);
}
```

---

## Performance Notes

| Aspect | Detail |
| :----- | :----- |
| **Execution frequency** | Called once per node per candidate path — keep logic lightweight |
| **Effect of pruning** | Reduces exponential branching factor — aggressive pruning dramatically speeds up speculative search |
| **Parallel execution** | Multiple path explorers run concurrently; each has its own `ctx.path_so_far` |

---

## Related

- [`on_path_resolve`](./on_path_resolve.md) — Fires when the winning path is found
- [`on_traverse`](./on_traverse.md) — Edge-level hook during regular graph traversal
- [Speculative Graph Routing](../../architecture/speculative-routing.md) — Architecture overview
