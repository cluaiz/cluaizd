---
title: join
description: Perform an in-memory hash join across disparate data models.
---

# `join` Command

The `join` command allows querying relational patterns across different document types by performing an in-memory Hash Join.

## Syntax
```cdql
-> join(target: "<label>", on_left: "<field1>", on_right: "<field2>", type: "<join_type>")
```

## Parameters
- `target` (String): The target neuron label to join against.
- `on_left` (String): The field on the current working set to match.
- `on_right` (String): The corresponding field on the target set to match.
- `type` (Optional String): The type of join (`inner`, `left`, `right`, `full`). Default is `inner`.

## Example
Find users and only keep those that have an existing order record matching their ID:
```cdql
find User(*) -> join(target: "Order", on_left: "id", on_right: "user_id", type: "inner")
```
