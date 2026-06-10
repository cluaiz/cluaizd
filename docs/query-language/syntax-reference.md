# CDQL Syntax & Operators — Complete Reference

> All operators, all parameters, all return types in one place.

---

## Root Selectors (Every Query Starts Here)

### `find id("uuid")` — O(1) Fast-Path ⚡
Bypasses CDQL Planner and WASM entirely. Single LMDB key lookup.
```text
find id("user_aryan_001")
```
- **Latency:** `< 0.1ms`
- **When to use:** Any time you know the exact neuron ID.

### `find Label(field: value, ...)` — Filtered Scan
Scans all neurons in the shard with the given label and filters.
```text
find User(status: "active", role: "admin")
```
- `Label`: Logical grouping name (stored in payload JSON).
- `field: value`: Equality filter applied during scan.

### `find *(field: value, ...)` — Wildcard Scan
Scans ALL neurons regardless of label.
```text
find *(tier: "Hot") -> limit 100
```

---

## Filter & Shape Operators

### `filter` — Narrow the Working Set
```text
-> filter age > 18
-> filter age >= 18
-> filter age < 65
-> filter age != 0
-> filter status: "active"             # Equality
-> filter tags contains "premium"      # Array contains
-> filter time between "now-1h" and "now"  # Time range
```

### `sort_by` — Order Results
```text
-> sort_by("created_at", asc: false)   # Newest first
-> sort_by("name", asc: true)          # A-Z
-> sort_by_score()                     # By BM25/cosine score (after search/similar_to)
-> sort_by_distance(to: [28.6, 77.2])  # By geo distance
```

### `limit` — Cap Result Count
```text
-> limit 10
-> limit 1000
```
> [!TIP]
> Always use `limit` with graph traversals and vector searches to prevent unbounded result sets.

### `project` — Select Specific Fields
```text
-> project(keep: ["name", "email", "created_at"])
```

### `unwind` — Expand Array Fields
```text
-> unwind("tags")    # Each tag becomes a separate result row
```

---

## Graph Operators

### `traverse` — Follow Adjacency Edges
```text
-> traverse(edge: "friends", hops: 1..1)         # Direct friends
-> traverse(edge: "friends", hops: 1..5)         # Up to 5 hops
-> traverse(edge: "friends", min_weight: 0.8)    # Strong connections only
-> traverse(hops: 1..3)                          # All edge types
```

| Parameter | Type | Description |
|---|---|---|
| `edge` | `string` | Filter by relation type. Omit for all edges. |
| `hops` | `N..M` | Min and max traversal depth. |
| `min_weight` | `float` | Minimum edge weight threshold (0.0 - 1.0). |

### `shortest_path` — Dijkstra Path Finding *(Planned)*
```text
-> shortest_path(to: "node_b_uuid")
```

---

## Relational Operators

### `join` — In-Memory Hash Join
```text
-> join(target: "Order", on_left: "id", on_right: "user_id", type: "inner")
-> join(target: "Product", on_left: "product_id", on_right: "id", type: "left")
```

| Parameter | Type | Description |
|---|---|---|
| `target` | `string` | Label of neurons to join with. |
| `on_left` | `string` | Field to match on the current working set. |
| `on_right`| `string` | Field to match on the target dataset. |
| `type` | `"inner"` \| `"left"` \| `"right"` \| `"full"` | Join type. |

### `group_by` — Bucket Records by Field
```text
-> group_by("department")
-> group_by("category")
```

### `aggregate` — Compute Stats on Groups
```text
-> aggregate(count())
-> aggregate(sum(price_usd))
-> aggregate(avg(rating))
-> aggregate(max(temperature))
-> aggregate(min(temperature))
-> aggregate(count(), sum(price_usd), avg(rating))  # Multiple at once
```

---

## Vector AI Operators

### `similar_to` — Semantic Similarity Search
```text
-> similar_to(vector: [0.12, -0.44, 0.89, ...], metric: "cosine")
-> similar_to(vector: [...], metric: "l2")
-> similar_to(vector: [...], metric: "dot")
```

| Metric | Best For |
|---|---|
| `"cosine"` | Text and semantic embeddings (normalized vectors) |
| `"l2"` | Image similarity, numerical feature spaces |
| `"dot"` | Recommendation systems (Max Inner Product) |

---

## Full-Text Search Operators

### `search` — BM25 Inverted Index Search
```text
-> search(query: "pizza", fuzzy: false)
-> search(query: "pizaa", fuzzy: true)                              # Typo tolerance
-> search(fields: {title: 3.0, body: 1.0}, query: "rust database") # Field boosting
```

---

## Geo-Spatial Operators

### `geo_near` — Haversine Radius Search
```text
-> geo_near(lat: 28.6139, lon: 77.2090, radius_km: 50.0)
-> geo_near(lat: 12.9716, lon: 77.5946, radius_km: 2.0)
```
> **Note:** The parameter is `radius_km` (a plain `f64` number), NOT `radius: "5km"`. The value is always in kilometres.

### `geo_within` — Bounding Box or Polygon *(Planned)*
```text
-> geo_within(lat_min: 28.55, lat_max: 28.70, lon_min: 77.10, lon_max: 77.30)
```

---

## Time-Series Operators

### `time_window` — Bucket by Time
```text
-> time_window(size: "1m")    # 1-minute buckets
-> time_window(size: "5m")    # 5-minute buckets
-> time_window(size: "1h")    # Hourly buckets
-> time_window(size: "1d")    # Daily buckets
```

### `range` — Inclusive Field Boundary Scan
```text
-> range(field: "age",       start: 18,          end: 35)
-> range(field: "salary",    start: 60000,        end: 120000)
-> range(field: "city",      start: "Chennai",    end: "Pune")   # Lexicographic
-> range(field: "timestamp", start: 1717789200,   end: 1717792800)
```

| Parameter | Type | Description |
|---|---|---|
| `field` | `string` | JSON key to read from each neuron's payload |
| `start` | `number \| string` | Lower bound (inclusive) |
| `end` | `number \| string` | Upper bound (inclusive) |

> Both `start` and `end` must be the same type. Mix of number + string → no results.

---

## Blob & Object Operators

### `stream` — Byte-Range Streaming
```text
-> stream(bytes: 0..1048576)          # First 1MB
-> stream(bytes: 104857600..209715200) # Bytes 100MB-200MB
```

### `pluck_metadata` — Shell-Only Fetch
```text
-> pluck_metadata()    # Returns neuron shell without triggering Cold-tier decompression
```

---

## Operator Performance Quick Reference

| Operator | Complexity | Notes |
|---|---|---|
| `find id()` | `O(1)` | LMDB mmap direct |
| `filter` (exact) | `O(n)` | Scans working set |
| `range(...)` | `O(n)` | JSON field extract per neuron |
| `sort_by` | `O(n log n)` | In-memory sort |
| `traverse` 1 hop | `O(edges)` | Native Graph Engine BFS |
| `traverse` N hops | `O(edges^N)` | Native Graph Engine BFS (Use `limit`!) |
| `join` | `O(n × m)` | Hash join |
| `similar_to` | `< 1ms` | MV-HNSW Approximate Nearest Neighbor |
| `time_window`| `< 2ms` | Gorilla compression bucketing |
| `search` (exact) | `O(n × tokens)` | Token match per neuron |
| `search` (fuzzy) | `O(n × tokens × words)` | Substring scan |
| `geo_near` | `O(n)` | Haversine per candidate, sorted by proximity |
