---
title: stream
description: Byte-range slicing for blob/object storage in CDQL.
---

# `stream` Command

The `stream` command slices the `raw_payload` bytes of the neurons in the working set. It is used for streaming partial content (e.g., video streaming or chunked file downloads).

## Syntax
```cdql
-> stream(start: <int>, end: <int>)
```

## Parameters
- `start` (Optional Integer): The starting byte offset. Default is `0`.
- `end` (Optional Integer): The ending byte offset. Default is the end of the file.

## Example
Fetch the first 1024 bytes of a specific PDF blob:
```cdql
find Blob(id: "doc123") -> stream(start: 0, end: 1024)
```
