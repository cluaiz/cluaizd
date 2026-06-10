---
title: traverse
description: Multi-hop graph traversal command in CDQL.
---

# `traverse` Command

The `traverse` command performs a multi-hop Breadth-First Search (BFS) starting from the current working set, following adjacency edges to discover connected neurons.

## Syntax
```cdql
-> traverse(edge: "<label>", min_hops: <int>, max_hops: <int>, min_weight: <float>)
```

## Parameters
- `edge` (Optional String): Filter edges by the target neuron's label or type. Default is `"*"` (follow all edges).
- `min_hops` (Optional Integer): The minimum path depth to include. Default is `1`.
- `max_hops` (Optional Integer): The maximum depth to traverse. Default is `3`.
- `min_weight` (Optional Float): Minimum edge weight required to follow the connection. Default is `0.0`.

## Example
Find a specific user, then traverse out to their "friends" up to 2 hops away:
```cdql
find User(id: "123") -> traverse(edge: "friends", max_hops: 2)
```
