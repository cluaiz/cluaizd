# `geo_near` — Geo-Spatial Proximity Reference

The `geo_near` pipeline stage executes a **great-circle distance radius filter** against every neuron in the working set.
It reads latitude/longitude coordinates from each neuron's JSON payload, computes the Haversine distance to a target point, and retains only neurons within the specified radius — sorted from closest to furthest.

---

## Syntax

```text
find * -> geo_near(lat: <f64>, lon: <f64>, radius_km: <f64>)
```

| Parameter   | Type  | Required | Description |
| :---------- | :---- | :------- | :---------- |
| `lat`       | `f64` | ✅ Yes   | Target point latitude in decimal degrees (WGS-84). |
| `lon`       | `f64` | ✅ Yes   | Target point longitude in decimal degrees (WGS-84). |
| `radius_km` | `f64` | ✅ Yes   | Search radius in kilometres. Neurons beyond this distance are excluded. |

---

## Coordinate Key Convention

The engine probes each neuron's JSON payload for coordinate fields in this priority order:

| Coordinate | Keys probed (in order) |
| :--------- | :--------------------- |
| Latitude   | `"lat"` → `"latitude"` |
| Longitude  | `"lon"` → `"longitude"` |

A neuron is automatically excluded if **neither** key variant is found.

### Accepted Payload Formats

```json
{ "lat": 28.6139, "lon": 77.2090 }
```
```json
{ "latitude": 28.6139, "longitude": 77.2090, "city": "New Delhi" }
```
```json
{ "name": "Sensor A", "lat": 12.9716, "lon": 77.5946, "temperature": 38.5 }
```

---

## Architecture: How It Works Under the Hood

The `geo_near` step is evaluated inside [`crates/genome/src/cdql/eval.rs`](../../../../crates/genome/src/cdql/eval.rs) by the `eval_geo_near()` function.

### Execution Pipeline

```
CDQL string
   │
   ▼
 parse()        ← crates/genome/src/cdql/parser.rs
   │  Produces CdqlQuery AST
   ▼
 build_plan()   ← crates/genome/src/cdql/planner.rs
   │  Emits PlanStep::GeoNear { lat, lon, radius_km }
   ▼
 execute_cdql() ← crates/server/src/routes/query.rs
   │  Scores each neuron via eval_geo_near()
   ▼
 eval_geo_near(payload, target_lat, target_lon, radius_km)
   │  ← crates/genome/src/cdql/eval.rs
   │
   ├── 1. serde_json::from_str(payload)
   │       Parse JSON. Returns None on failure — no panic.
   │
   ├── 2. probe_f64(&json, &["lat", "latitude"])
   │   probe_f64(&json, &["lon", "longitude"])
   │       Returns None if neither key is present.
   │
   ├── 3. haversine_km(neuron_lat, neuron_lon, target_lat, target_lon)
   │       Computes great-circle distance on the WGS-84 Earth ellipsoid.
   │
   ├── 4. if dist_km <= radius_km:
   │       score = 1.0 / (1.0 + dist_km)   ← inverse distance score ∈ (0, 1]
   │       return Some(score)
   │   else:
   │       return None  ← neuron is excluded
   │
   ▼
 Results sorted descending by score (closest neuron first)
```

### Haversine Formula (Internal Implementation)

The engine uses the **Haversine formula** to compute great-circle distance on a spherical Earth (R = 6371 km):

```
Δφ = lat₂ − lat₁   (in radians)
Δλ = lon₂ − lon₁   (in radians)

a = sin²(Δφ/2) + cos(lat₁) · cos(lat₂) · sin²(Δλ/2)
c = 2 · atan2(√a, √(1−a))
d = R · c
```

This formula is accurate to within ~0.3% for distances up to 1,000 km and is computed in pure Rust with no external dependencies.

### Proximity Score

Each neuron inside the radius receives a **proximity score**:

```
score = 1.0 / (1.0 + distance_km)
```

| Distance (km) | Score |
| :------------ | :---- |
| 0 km (exact location) | `1.000` |
| 10 km | `0.091` |
| 35 km | `0.028` |
| 50 km | `0.020` |
| > radius | `None` → **excluded** |

Results are returned **sorted descending by score** — closest location first.

---

## Time Complexity

| Scenario | Complexity | Notes |
| :------- | :--------- | :---- |
| Unindexed geo scan | **O(N)** | Haversine is computed for every neuron in the working set |
| After upstream `-> range(...)` | **O(M)** | M = survivors from previous stage; reduces scan cost |

> **Performance Tip**: Pre-filter with `-> range(...)` on a bounding-box field (e.g., a city code or region ID) before `-> geo_near(...)` to dramatically shrink the working set.

---

## Real-World Accuracy Reference

| City pair | Haversine result | Actual distance |
| :-------- | :--------------- | :-------------- |
| Delhi → Gurgaon | ~32.8 km | ~30 km (road) |
| Delhi → Mumbai | ~1,148 km | ~1,150 km (air) |
| Bengaluru → Chennai | ~290 km | ~290 km (air) |

---

## Examples

### 1. Find All Locations Within 50 km of New Delhi

```json
{
  "tenant_id": "my_tenant",
  "cdql": "find * -> geo_near(lat: 28.6139, lon: 77.2090, radius_km: 50.0)"
}
```

---

### 2. Find Nearby Restaurants (Compact Radius)

```json
{
  "tenant_id": "restaurant_db",
  "cdql": "find * -> geo_near(lat: 12.9716, lon: 77.5946, radius_km: 2.0)"
}
```
Finds all restaurants within 2 km of a given point in Bengaluru.

---

### 3. Chained with `range` (Stars + Distance)

```json
{
  "tenant_id": "hotel_db",
  "cdql": "find * -> geo_near(lat: 19.0760, lon: 72.8777, radius_km: 15.0) -> range(field: \"stars\", start: 4, end: 5)"
}
```
Returns 4–5 star hotels within 15 km of Mumbai.

---

### 4. Chained with `search` (Nearby + Keyword)

```json
{
  "tenant_id": "property_db",
  "cdql": "find * -> geo_near(lat: 28.6139, lon: 77.2090, radius_km: 25.0) -> search(query: \"metro connectivity\", fuzzy: true)"
}
```
Finds properties within 25 km of Delhi that mention metro connectivity.

---

### 5. Using `latitude` / `longitude` Key Variants

Neurons stored with full key names are automatically resolved:

```json
{ "latitude": 28.4595, "longitude": 77.0266, "name": "Gurgaon Office" }
```

No special configuration needed — the engine probes both variants automatically.

---

## API Request Shape

`POST /query`

```json
{
  "tenant_id": "my_tenant",
  "cdql": "find * -> geo_near(lat: <f64>, lon: <f64>, radius_km: <f64>)"
}
```

### Response

```json
[
  {
    "neuron": {
      "id": "uuid-...",
      "raw_payload": { "city": "Gurgaon", "lat": 28.4595, "lon": 77.0266 }
    }
  },
  {
    "neuron": {
      "id": "uuid-...",
      "raw_payload": { "city": "Noida", "latitude": 28.5355, "longitude": 77.3910 }
    }
  }
]
```

> Results are ordered **closest first**. No additional `-> sort` stage is needed.

---

## Error Handling

| Condition | Behaviour |
| :-------- | :-------- |
| Neuron payload has no `lat`/`latitude` key | Neuron **excluded** silently |
| Neuron payload has no `lon`/`longitude` key | Neuron **excluded** silently |
| Coordinate value is not a valid number | Neuron **excluded** silently — no panic |
| Payload is not valid JSON | Neuron **excluded** silently |
| `radius_km` is `0.0` | Only neurons at the exact target point are included (distance = 0) |
| `radius_km` is negative | No neurons match — empty result |

---

## Related

- [`search`](./search.md) — Full-text keyword search
- [`range`](./range.md) — Numeric or lexicographic range filtering
- [`euclidean_distance`](./euclidean_distance.md) — Flat-plane L2 distance for vector spaces
- [`find`](./find.md) — Base traversal command
- [`limit`](./limit.md) — Result count truncation
- [CDQL Advanced Pipelines](../../cdql/advanced-pipelines.md) — Chaining multiple pipeline stages
