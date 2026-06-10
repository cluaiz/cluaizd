---
title: aggregate
description: Perform data aggregations on the working set.
---

# `aggregate` Command

The `aggregate` command computes aggregate values (like totals, averages, and counts) on the current working set. It is typically used after `group_by` or `time_window`.

## Syntax
```cdql
-> aggregate(<func1>, <func2>, ...)
```

## Functions Supported
- `count()`: Total number of neurons in the bucket.
- `sum("<field>")`: Sum of a numeric field.
- `avg("<field>")`: Average of a numeric field.
- `min("<field>")`: Minimum value of a numeric field.
- `max("<field>")`: Maximum value of a numeric field.

## Example
Count the number of items and sum their values per category:
```cdql
find Items(*) -> group_by("category") -> aggregate(count(), sum("price"))
```

Note: Executing an `aggregate` command short-circuits the pipeline and returns a flat JSON array of aggregate results rather than standard `QueryResult` objects.
