---
title: unwind
description: Explode an array field into multiple result rows.
---

# `unwind` Command

The `unwind` command flattens an array field within a neuron's JSON payload. For each element in the specified array, a duplicate neuron reference is emitted into the working set. If the field is not an array or does not exist, the neuron is passed through as-is.

## Syntax
```cdql
-> unwind("<field>")
```

## Parameters
- `field` (String): The name of the JSON array field to unwind.

## Example
If a blog post has multiple tags, explode the tags array so each tag gets its own result row:
```cdql
find Post(*) -> unwind("tags")
```
