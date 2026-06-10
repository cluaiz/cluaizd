---
title: time_window
description: Bucket time-series data by time intervals.
---

# `time_window` Command

The `time_window` command groups time-series data into discrete temporal buckets based on a specified time duration. It requires the neurons to have a `timestamp`, `ts`, `time`, or `created_at` field (or uses the internal creation time).

## Syntax
```cdql
-> time_window(size: "<duration>")
```

## Parameters
- `size` (String): The duration of the window (e.g., `"1s"`, `"5m"`, `"1h"`, `"1d"`).

## Example
Group server logs into 1-hour buckets and count errors:
```cdql
find Logs(type: "error") -> time_window(size: "1h") -> aggregate(count())
```
