---
title: project
description: Shape documents by keeping specific JSON fields.
---

# `project` Command

The `project` command filters the JSON fields of the neurons in the working set, returning only the specified fields. Neurons that do not contain any of the specified fields will be excluded from the results.

## Syntax
```cdql
-> project("field1", "field2", ...)
```

## Parameters
- `keep` (Variable Strings): The JSON fields to retain.

## Example
Return only the `name` and `email` of all users:
```cdql
find User(*) -> project("name", "email")
```
