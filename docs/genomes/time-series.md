# Time-Series Genome (`time_series.json`)

> *"Every second is a data point. Every hour is a story."*

## When to Use This Genome
Use the `time_series` genome when:
- Data is generated continuously with timestamps (logs, metrics, telemetry).
- You need to aggregate data over time windows (hourly averages, daily sums).
- Old data should automatically compress to save storage.
- Write throughput is very high (thousands of events/second).

Real-world use cases: Server monitoring (CPU/RAM metrics), IoT sensor readings, financial tick data, application error logs, user activity analytics, BCI neural recordings.

---

## How Time-Series Data Flows

```
Sensor/Device
     │
     ▼ (Write: must include `timestamp`)
  on_write hook validates timestamp exists
     │
     ▼
  LMDB stores the raw Neuron (Hot Tier)
     │
     ▼ (Background: Dreamer thread evaluates on_lifecycle)
  After cold_ttl_ns → ZSTD compress to Cold Tier
```

### The `time_series.json` Genome
```json
{
  "on_write": "let res = #{ action: \"Allow\" };\nif !payload.contains(\"timestamp\") {\n    res.action = \"Abort\";\n    res.error = \"Telemetry must include timestamp\";\n}\nres",
  "on_lifecycle": "let res = #{};\nif age_ns > config.cold_ttl_ns {\n    res.new_tier = \"Cold\";\n    res.compress_level = 5;\n}\nres",
  "parameters": { "cold_ttl_ns": 3600000000000 },
  "engine": "rhai"
}
```

Records older than 1 hour (`3,600,000,000,000` nanoseconds) are automatically moved to ZSTD-compressed Cold storage. No manual archival scripts needed.

---

## Writing Telemetry Data

```bash
curl -X POST http://localhost:7331/data \
  -H "Content-Type: application/json" \
  -d '{
    "id": "reading_1717789200001",
    "tier": "Hot",
    "raw_payload": [bytes for: {"sensor_id": "temp_01", "timestamp": 1717789200, "value": 45.2, "unit": "celsius"}],
    "vector_data": [],
    "adjacency": []
  }'
```

> [!NOTE]
> For maximum throughput, use the C-FFI bindings instead of HTTP. The C-FFI can ingest over 1 million writes/second, whereas HTTP is typically limited by TCP overhead to ~50,000 writes/second.

---

## Time-Range Filtering

```text
// Get all sensor readings from the last 1 hour
find Sensor(sensor_id: "temp_01") 
  -> filter time between "now-1h" and "now"
  -> limit 3600
```

---

## Time-Window Aggregation (InfluxDB `GROUP BY time()` Equivalent)

```text
// Average temperature per 5-minute window over the last 6 hours
find Sensor(sensor_id: "temp_01")
  -> filter time between "now-6h" and "now"
  -> time_window(size: "5m")
  -> aggregate(avg(value), min(value), max(value))
```

This is equivalent to:
```sql
SELECT 
  time_bucket('5 minutes', timestamp) AS bucket,
  AVG(value), MIN(value), MAX(value)
FROM sensor_readings
WHERE sensor_id = 'temp_01' AND timestamp > NOW() - INTERVAL '6 hours'
GROUP BY bucket
ORDER BY bucket;
```

---

## Downsampling Strategy (Long-Term Storage)

As data ages, granularity becomes less important. Instead of storing every 1-second reading for a year, you can write a custom lifecycle genome that downsamples:

```json
{
  "on_lifecycle": "
    let res = #{};
    if age_ns > 86400000000000 {
      // After 24 hours: move to Cold, compress level 9
      res.new_tier = 'Cold';
      res.compress_level = 9;
    }
    res
  "
}
```

Paired with periodic CNQL aggregation jobs that write hourly summaries, you can store months of historical data on a $5/month server.

---

## Comparison: CNSDB vs InfluxDB

| Feature | InfluxDB | CNSDB (time_series) |
|---|---|---|
| Timestamp Enforcement | ✅ | ✅ |
| Time-Window Aggregations | ✅ | ✅ (via CNQL) |
| Automatic Downsampling | ✅ | ✅ (via Genome lifecycle) |
| Vector Search on Readings | ❌ | ✅ (hybrid genome) |
| Graph Relationships | ❌ | ✅ (via adjacency) |
| Cost for 1TB/year | ~$400/mo | ~$5/mo (with ZSTD Cold) |
