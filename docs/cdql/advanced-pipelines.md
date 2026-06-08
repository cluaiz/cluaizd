# Advanced CDQL Pipelines

> *"One pipeline. Ten databases. Zero compromises."*

## The Pipeline Mental Model

Think of CDQL like a Unix pipe (`|`). Data flows from left to right through a series of transformation stages. Each stage receives the output of the previous stage, processes it, and passes results forward.

```
find User(active: true) -> traverse(edge: "friends") -> filter age > 25 -> limit 50
         │                          │                          │                │
     (Start: Load            (Graph: follow         (Filter: narrow        (Stop: cap
      matching neurons)        adjacency edges)       the result set)        results)
```

The power of CDQL is that these stages can span entirely different database paradigms — all within the same memory space, with zero network hops.

---

## Example 1: The "People You May Know" Recommendation

*Classic LinkedIn-style: find users who are friends-of-friends, share similar interests (vector), and live nearby (geo).*

```text
find User(status: "active")
  -> traverse(edge: "friends", hops: 2..2)      // Friends of friends only (not direct friends)
  -> filter already_connected: false             // Exclude people you already know
  -> geo_near(lat: 28.61, lon: 77.20, radius: "25km")  // Must be local
  -> similar_to(vector: [my_interest_embedding], metric: "cosine")  // Share similar interests
  -> sort_by_score()
  -> limit 20
```

In a Database Zoo this requires: Graph DB query → join with Relational DB → PostGIS filter → Vector DB query → Python merge sort. That is 5 API calls. CLUAIZD does it in 1.

---

## Example 2: Anomaly Detection on Time-Series IoT Data

*Find sensors that are reading abnormally, have spiked in the last 10 minutes, and are in a specific building.*

```text
find Sensor(building: "Block_C", status: "active")
  -> filter time between "now-10m" and "now"
  -> time_window(size: "1m")
  -> aggregate(avg(value), max(value))
  -> filter max_value > 95.0
  -> sort_by("max_value", asc: false)
  -> limit 10
```

---

## Example 3: E-Commerce Fraud Detection

*Find recent orders from high-risk users, where the product matches a known fraud pattern, placed from a suspicious IP range.*

```text
find Order(created_after: "now-24h")
  -> join(target: "User", on: "user_id == target.id", type: "inner")
  -> filter target.risk_score > 0.7
  -> join(target: "Product", on: "product_id == target.id", type: "inner")
  -> similar_to(vector: [fraud_product_embedding], metric: "cosine")
  -> filter target.price_usd > 500
  -> sort_by("target.risk_score", asc: false)
  -> limit 50
```

---

## Example 4: AI Agent Memory Retrieval with Episodic + Semantic Layers

*The agent needs to recall memories that are both semantically relevant AND happened recently.*

```text
find Memory(agent_id: "agent_007")
  -> filter time between "now-7d" and "now"          // Only the last week (Episodic layer)
  -> similar_to(vector: [query_embedding], metric: "cosine")  // Semantic relevance (Semantic layer)
  -> sort_by_score()
  -> limit 5
```

This is the hybrid episodic + semantic memory retrieval that LLM agents need to behave like humans.

---

## Example 5: Location-Aware Full-Text Search with Ranking

*Find restaurants within 3km with "butter chicken" in the menu, ranked by relevance score, with a rating filter.*

```text
find Restaurant(open_now: true, rating >= 4.0)
  -> geo_near(lat: 28.61, lon: 77.20, radius: "3km")
  -> search(fields: {menu: 2.0, name: 3.0}, query: "butter chicken", fuzzy: true)
  -> sort_by_score()
  -> limit 20
```

---

## Example 6: Graph Fraud Ring Detection with Blob Evidence

*Find all accounts in a fraud ring (graph traversal), then retrieve their transaction receipt blobs (byte-range stream).*

```text
// Step 1: Find the fraud ring
find Account(id: "known_fraud_X")
  -> traverse(edge: "shares_device", hops: 1..4)
  -> filter risk_score > 0.6

// Step 2: For each fraud account, stream their transaction evidence
find id("fraudster_account_Y") -> stream(bytes: 0..4096)
```

---

## Combining Wide-Column Scan with Vector Search

*Replay BCI neural recordings from the last hour and find the 10 most anomalous readings by vector distance from the normal baseline.*

```text
find BCIReading(electrode: 42)
  -> range_scan(field: "timestamp", start: "now-1h", end: "now")
  -> similar_to(vector: [normal_baseline_vector], metric: "l2")
  -> sort_by_score(ascending: false)   // Most DIFFERENT first (anomaly detection)
  -> limit 10
```

---

## Pipeline Performance Tips

> [!TIP]
> **Always filter early.** Put the most selective filter stages first to reduce the working set before expensive operations like `traverse()` or `similar_to()`.
> 
> ✅ `find User(active: true) -> traverse() -> similar_to()`
> ❌ `find User -> similar_to() -> filter active: true` *(scans ALL users before filtering)*

> [!TIP]
> **Use `find id()` for single-record lookups.** This triggers the 0ms Fast-Path, bypassing WASM entirely.
