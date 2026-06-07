# Geo-Spatial Genome (`geospatial.json`)

> *"The universe has coordinates. So should your data."*

## When to Use This Genome
Use the `geospatial` genome when:
- Your data has a physical location (latitude, longitude, altitude).
- You need to find the nearest neighbors to a GPS coordinate.
- You need to check if a point is within a geographic boundary (polygon, city, country).
- You are building location-aware features.

Real-world use cases: "Find restaurants near me", delivery driver tracking, geofencing alerts, real estate search by neighborhood, logistics route optimization, store locator, disaster relief resource mapping.

---

## The Mathematics of Earth-Aware Distances

### Why Not Just Use Euclidean Distance?
Earth is not flat. Two points at `lat: 28.6, lon: 77.2` and `lat: 28.7, lon: 77.3` are NOT simply 0.1 units apart. Because Earth is a sphere, distance along the surface follows a curved path.

Using standard Euclidean math near the poles would give wildly incorrect distances.

### The Haversine Formula
CLUAIZD uses the **Haversine Formula** to compute the great-circle distance between two points on Earth's surface:

```
a = sin²(Δlat/2) + cos(lat₁)·cos(lat₂)·sin²(Δlon/2)
c = 2·atan2(√a, √(1−a))
distance = 6371 km × c   (Earth's radius = 6371 km)
```

This gives the true shortest path distance along Earth's curved surface — accurate to within meters at any point on the globe.

---

## Storing Location Data

Every Neuron that needs geo-spatial capabilities must store its coordinates in the `raw_payload` JSON:

```bash
curl -X POST http://localhost:7331/data \
  -H "Content-Type: application/json" \
  -d '{
    "id": "restaurant_001",
    "tier": "Hot",
    "raw_payload": [bytes for:
      {
        "name": "Delhi Darbar",
        "cuisine": "Indian",
        "rating": 4.8,
        "price_tier": "budget",
        "lat": 28.6139,
        "lon": 77.2090,
        "open_now": true
      }
    ],
    "vector_data": [],
    "adjacency": []
  }'
```

---

## Radius Search (The Most Common Geo Query)

Find all restaurants within 5km of a user's location:

```text
find Restaurant(open_now: true)
  -> geo_near(lat: 28.6200, lon: 77.2100, radius: "5km")
  -> sort_by_distance(to: [28.6200, 77.2100])
  -> limit 20
```

This returns only restaurants within a 5km radius, sorted from nearest to farthest. The `geo_near` step calculates the Haversine distance for every candidate Neuron and filters out those exceeding the radius.

---

## Bounding Box Search

For faster (but less precise) spatial queries, you can use a rectangular bounding box instead of a radius. This is useful for "all restaurants on this map viewport":

```text
// All restaurants within a map viewport bounding box
find Restaurant
  -> geo_within(lat_min: 28.55, lat_max: 28.70, lon_min: 77.10, lon_max: 77.30)
  -> limit 100
```

Bounding box queries do NOT use Haversine — they use simple coordinate comparisons (`O(n)`) which is 10x faster but returns a rectangular region rather than a circular radius.

---

## Combining Geo with Other Paradigms

### Geo + Full-Text (Find Indian Restaurants Near Me)
```text
find Restaurant
  -> geo_near(lat: 28.6139, lon: 77.2090, radius: "3km")
  -> search(query: "butter chicken", fuzzy: true)
  -> sort_by_score()
  -> limit 10
```

### Geo + Vector AI (Semantic Recommendation Near Me)
```text
// Find restaurants semantically similar to "romantic candlelight dinner" within 10km
find Restaurant(open_now: true)
  -> geo_near(lat: 28.6139, lon: 77.2090, radius: "10km")
  -> similar_to(vector: [romantic_dinner_embedding], metric: "cosine")
  -> limit 5
```

---

## Geofencing Alerts (Delivery Tracking)

A delivery driver enters a geofence (500m from a warehouse). Trigger an alert:

```text
// Check if driver is within 500m of depot
find Driver(id: "driver_aryan")
  -> geo_near(lat: 28.6139, lon: 77.2090, radius: "0.5km")
```

If the query returns a result, the driver is inside the geofence. If empty, they are outside. Poll this every 10 seconds from your backend.

---

## Comparison: CLUAIZD vs PostGIS

| Feature | PostGIS | CLUAIZD (geospatial) |
|---|---|---|
| Haversine Distance | ✅ | ✅ |
| Bounding Box Filter | ✅ | ✅ |
| Polygon Containment | ✅ | 🔜 (Planned) |
| Spatial Indexes (R-Tree) | ✅ | 🔜 (Planned — currently linear scan) |
| Graph Traversal from Points | ❌ | ✅ |
| Vector Similarity on Points | ❌ | ✅ |
| Requires PostgreSQL + Extension | ✅ | ❌ (standalone) |
