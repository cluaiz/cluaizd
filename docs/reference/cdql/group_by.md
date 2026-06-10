---
title: group_by
description: Bucket results into groups based on specified fields.
---

# `group_by` Command

The `group_by` command groups the current working set into buckets based on one or more JSON fields. It is always used immediately before the `aggregate` command.

## Syntax
```cdql
-> group_by("field1", "field2", ...)
```

## Parameters
- `fields` (Variable Strings): The JSON fields to group by.

## Example
Group employees by department:
```cdql
find Employee(*) -> group_by("department") -> aggregate(count())
```
