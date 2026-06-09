# REST API Complete Reference

> **Base URL:** `http://localhost:7331`  
> **Server:** Axum (Rust async web framework)  
> **Content-Type:** `application/json` for all endpoints

---

## Complete Route Table

| Method | Endpoint | Description |
|---|---|---|
| `POST` | `/neuron` | Insert or update a UniversalNeuron |
| `GET` | `/neuron/{id}` | Fetch a single neuron by UUID |
| `POST` | `/query` | Execute a CDQL query pipeline |
| `GET` | `/stream/{id}` | Zero-copy byte-range media streaming |
| `GET` | `/graph/{id}/traverse` | Deep graph traversal from a node |
| `GET` | `/juju/state` | Live spatial map of all neurons and edges |
| `POST` | `/crispr/clamp/{id}` | Lock a DNA parameter to a fixed value |
| `POST` | `/crispr/force/{id}` | Inject a permanent synaptic edge |
| `POST` | `/booster/upload` | Upload a custom WASM compute engine |
| `POST` | `/booster/mode/{mode}` | Switch booster performance mode |
| `POST` | `/ingest/stream` | High-throughput sensory stream ingestion |
| `POST` | `/sandbox/validate` | Validate mutations in Deep Archer sandbox |
| `POST` | `/dna/setup` | Register new DNA Execution code (Auto-WASM, Rhai, CDQL) |
| `GET` | `/ws/telemetry` | WebSocket live HEART telemetry + controls |

---

## `POST /neuron` — Write a Neuron

Inserts a new neuron or overwrites an existing one (same `id` = upsert).

**Headers:**
- `x-tenant-id` (optional): Routes to a specific LMDB shard. Default: `default_sandbox`

**Request Body — Full Schema:**
```json
{
  "id": "user_aryan_001",
  "tier": "Hot",
  "raw_payload": [123, 34, 110, 97, 109, 101, 34, 58, 32, 34, 65, 114, 121, 97, 110, 34, 125],
  "vector_data": [0.12, -0.44, 0.89, 0.33],
  "adjacency": [
    { "target_id": "user_bob_002", "weight": 0.95 }
  ],
  "dna": {
    "engine": "rhai",
    "on_write": "let res = #{ action: \"Allow\" }; res",
    "on_read": null,
    "on_index": null,
    "on_lifecycle": null,
    "parameters": { "ttl_ns": 3600000000000 },
    "wasm_module": null
  }
}
```

**Field Reference:**

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | `string` | ✅ | Unique identifier (any string, UUID recommended) |
| `tier` | `"Hot"` \| `"Warm"` \| `"Cold"` | ✅ | Initial storage tier |
| `raw_payload` | `number[]` | ✅ | Raw bytes (JSON encoded as UTF-8 byte array) |
| `vector_data` | `number[]` | ❌ | Float32 embedding array for vector search |
| `adjacency` | `object[]` | ❌ | Graph edges to other neurons |
| `dna` | `object` | ❌ | Genome script to attach to this neuron |

**Response: `201 Created`**
```json
{ "status": "written", "id": "user_aryan_001" }
```

**Shard Routing:**
```bash
# Write to the main shard (default)
curl -X POST http://localhost:7331/neuron \
  -H "Content-Type: application/json" \
  -d '{"id": "user_001", "tier": "Hot", "raw_payload": [...]}'

# Write to an isolated IoT shard
curl -X POST http://localhost:7331/neuron \
  -H "Content-Type: application/json" \
  -H "x-tenant-id: sensory_tissue" \
  -d '{"id": "sensor_reading_001", "tier": "Hot", "raw_payload": [...]}'
```

---

## `GET /neuron/{id}` — Read a Single Neuron

Returns the full neuron by its ID.

```bash
curl http://localhost:7331/neuron/user_aryan_001
```

**Response: `200 OK`**
```json
{
  "id": "user_aryan_001",
  "tier": "Hot",
  "raw_payload": [123, 34, 110, ...],
  "vector_data": [0.12, -0.44, 0.89],
  "adjacency": [],
  "dna": null
}
```

**Errors:**
- `404 Not Found` — Neuron ID does not exist in the shard.

---

## `POST /query` — Execute a CDQL Pipeline

Executes any CDQL query string against a shard.

```bash
curl -X POST http://localhost:7331/query \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "default_sandbox",
    "cdql": "find User(status: \"active\") -> traverse(edge: \"friends\") -> limit 10"
  }'
```

**Request Body:**

| Field | Type | Required | Description |
|---|---|---|---|
| `cdql` | `string` | ✅ | The CDQL pipeline query string |
| `tenant_id` | `string` | ❌ | Shard to query. Default: `default_sandbox` |

**Response: `200 OK`** — Array of matched neurons with scores:
```json
[
  {
    "score": 1.0,
    "matched_by": "cdql",
    "neuron": {
      "id": "user_bob_002",
      "tier": "Hot",
      "raw_payload": [...],
      "vector_data": [0.14, -0.41, 0.90],
      "adjacency": []
    }
  }
]
```

---

## `GET /stream/{id}` — Zero-Copy Byte-Range Streaming

Streams raw bytes of a neuron's payload without loading the entire payload into RAM. Uses HTTP Range headers.

```bash
# Stream the first 1MB of a stored video
curl -H "Range: bytes=0-1048575" http://localhost:7331/stream/video_drone_001
```

**Response:** Raw binary stream (`Content-Type: application/octet-stream`).

---

## `GET /graph/{id}/traverse` — Deep Graph Traversal

Traverses the adjacency graph starting from a given neuron.

```bash
# Traverse 3 hops from user_alice via "friends" edges
curl "http://localhost:7331/graph/user_alice/traverse?edge=friends&max_hops=3"
```

**Query Parameters:**

| Param | Type | Default | Description |
|---|---|---|---|
| `edge` | `string` | (all edges) | Filter by edge relation type |
| `max_hops` | `number` | `3` | Maximum traversal depth |
| `min_weight` | `number` | `0.0` | Minimum edge weight threshold |

---

## `POST /ingest/stream` — High-Throughput Sensor Ingestion

Bulk-write endpoint optimized for IoT/BCI stream ingestion. Routes directly to the `sensory_tissue` shard.

```bash
curl -X POST http://localhost:7331/ingest/stream \
  -H "Content-Type: application/json" \
  -d '[
    {"id": "reading_001", "tier": "Hot", "raw_payload": [...]},
    {"id": "reading_002", "tier": "Hot", "raw_payload": [...]}
  ]'
```

---

## `POST /sandbox/validate` — Deep Archer Sandbox

Validates proposed mutations (writes/deletes/genome changes) in an isolated in-memory sandbox before applying them to production data.

```bash
curl -X POST http://localhost:7331/sandbox/validate \
  -H "Content-Type: application/json" \
  -d '{
    "proposed_writes": [
      {"id": "test_neuron", "tier": "Hot", "raw_payload": [...]}
    ],
    "genome_to_test": "sql_strict.json"
  }'
```

**Response:**
```json
{ "valid": true, "errors": [], "warnings": [] }
```

---

## `POST /dna/setup` — Register DNA Execution Code

Registers new validation logic into the global Genome Registry. Supports Auto-WASM, CDQL, WASM, and Rhai.

```bash
curl -X POST http://localhost:7331/dna/setup \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my_custom_validator",
    "engine": "auto-wasm",
    "code": "use serde::Deserialize; ..."
  }'
```

**Request Body:**

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | `string` | ✅ | Unique name for this DNA sequence |
| `engine` | `string` | ✅ | `"auto-wasm"`, `"wasm"`, `"cdql"`, or `"rhai"` |
| `code` | `string` | ✅ | The raw code or Base64 binary |

**Response: `200 OK`**
```json
{ "status": "success", "message": "Auto-WASM DNA 'my_custom_validator' compiled successfully and hot-reloaded." }
```

---

## Error Reference

| HTTP Status | Meaning |
|---|---|
| `200 OK` | Success |
| `201 Created` | Neuron written successfully |
| `400 Bad Request` | Invalid JSON, missing required fields, or invalid UUID |
| `404 Not Found` | Neuron ID not found in the specified shard |
| `500 Internal Server Error` | LMDB error, disk full, or WAL write failure |
