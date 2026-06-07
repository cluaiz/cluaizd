# `POST /ingest/stream` — Raw Sensor Stream Ingestion

> *"No JSON. No schema. Just raw bytes — at maximum speed."*

## What Makes This Endpoint Different

`POST /ingest/stream` is NOT like `POST /neuron`. It accepts **raw binary bytes** (not JSON) and routes them directly to the isolated `sensory_tissue` shard — bypassing the main database entirely.

Key differences:

| Feature | `POST /neuron` | `POST /ingest/stream` |
|---|---|---|
| Body format | JSON | Raw binary bytes |
| Destination shard | Configurable (via `x-tenant-id`) | Always `sensory_tissue` |
| DNA hooks | Runs `on_write` genome hook | Uses `sensory_stream.json` genome |
| Pacemaker rate limit | Not applied | ✅ Applied (via WebSocket command) |
| TTL | Configurable | Fixed 30 seconds |

---

## Request

**Endpoint:** `POST /ingest/stream`  
**Content-Type:** `application/octet-stream`

**Headers:**

| Header | Required | Description |
|---|---|---|
| `Content-Type` | ✅ | Must be `application/octet-stream` |
| `X-Device-Id` | ❌ | Source device identifier for logging (e.g., `"bci_electrode_42"`) |

**Body:** Raw binary bytes — voltage readings, EEG samples, LiDAR point clouds, etc.

```bash
# Stream 8 bytes of raw sensor data
curl -X POST http://localhost:7331/ingest/stream \
  -H "Content-Type: application/octet-stream" \
  -H "X-Device-Id: bci_electrode_42" \
  --data-binary $'\x00\x01\xA4\xFF\x3C\x00\x7F\x10'
```

**Python example — stream a NumPy float32 array:**
```python
import numpy as np
import requests

# Simulate 256 electrode voltage readings
voltages = np.array([0.42, -0.11, 0.89, 0.33], dtype=np.float32)
raw_bytes = voltages.tobytes()

response = requests.post(
    "http://localhost:7331/ingest/stream",
    data=raw_bytes,
    headers={
        "Content-Type": "application/octet-stream",
        "X-Device-Id": "eeg_headset_01"
    }
)
print(response.json())
```

---

## Response: `202 Accepted`

```json
{
  "assigned_neuron_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "shard": "sensory_tissue",
  "ttl_ms": 30000
}
```

| Field | Description |
|---|---|
| `assigned_neuron_id` | Auto-generated UUID for this ingested frame |
| `shard` | Always `"sensory_tissue"` |
| `ttl_ms` | Time-to-live in milliseconds (30 seconds default) |

---

## Errors

| Status | Reason |
|---|---|
| `400 Bad Request` | Empty body |
| `413 Payload Too Large` | Body exceeds the Pacemaker rate limit set via WebSocket |
| `500 Internal Server Error` | Sensory shard write failure |

---

## Pacemaker Rate Limiting

The `POST /ingest/stream` endpoint respects the **Artificial Pacemaker** limit set via the WebSocket control channel. If you send a payload larger than the configured limit, you get a `413` response:

```json
{ "error": "Blocked by Pacemaker rate limit (max 1048576 bytes)" }
```

To change the limit, send via WebSocket:
```json
{ "command": "artificial_pacemaker", "payload": { "pulse_limit": 2097152 } }
```
Sets limit to 2 MB per ingestion call.

To disable the limit:
```json
{ "command": "artificial_pacemaker", "payload": { "pulse_limit": 0 } }
```
