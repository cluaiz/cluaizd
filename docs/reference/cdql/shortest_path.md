---
title: shortest_path
description: Find the shortest path between nodes in CDQL.
---

# `shortest_path` Command

The `shortest_path` command computes the shortest path using a Breadth-First Search (BFS) from the current working set neurons to a specific target neuron.

## Syntax
```cdql
-> shortest_path(to: "<target_id>")
```

## Parameters
- `to` (String): The unique ID of the target neuron.

## Example
Find the shortest connection path between User A and User B:
```cdql
find id("user_a") -> shortest_path(to: "user_b")
```
The result set will contain all neurons along the winning path.
