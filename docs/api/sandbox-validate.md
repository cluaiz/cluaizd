# `POST /sandbox/validate` — Deep Archer Sandbox

> *"Test before you commit. A sandbox that mirrors production — without touching it."*

## What is the Deep Archer Sandbox?

The Deep Archer Sandbox creates an **ephemeral in-memory copy** of a target neuron's LMDB environment. You submit a proposed mutation (new payload, new vector, new DNA), and the sandbox simulates whether this change would be **structurally safe** before you commit it to the WAL and LMDB.

This is critical for:
- **AI agents** that autonomously mutate neurons and need to verify safety before writing.
- **Schema migrations** — testing a new genome `on_write` hook against existing data.
- **Graph restructuring** — verifying that proposed edge changes won't cause structural imbalances.

---

## Request

**Endpoint:** `POST /sandbox/validate`  
**Content-Type:** `application/json`

**Headers:**

| Header | Required | Description |
|---|---|---|
| `x-tenant-id` | ❌ | Shard to validate against. Default: `default_sandbox` |

**Request Body — Full Schema:**

```json
{
  "original_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "proposed_payload": "{\"name\": \"Aryan Updated\", \"role\": \"super_admin\"}",
  "proposed_vector_data": [0.12, -0.44, 0.89, 0.33, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
  "model_creator_hash": "a3f9b2c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1",
  "payload_type": "text",
  "dna": {
    "engine": "rhai",
    "on_write": "let res = #{ action: \"Allow\" }; res",
    "on_read": null,
    "on_index": null,
    "on_lifecycle": null,
    "parameters": {}
  },
  "adjacency": [
    { "target_id": "f0e1d2c3-b4a5-9876-dcba-ef9876543210", "weight": 0.85 }
  ]
}
```

**Field Reference:**

| Field | Type | Required | Description |
|---|---|---|---|
| `original_id` | `UUID string` | ✅ | The existing neuron to simulate mutation on |
| `proposed_payload` | `string` | ✅ | New JSON payload as a string |
| `proposed_vector_data` | `[f32; 16]` | ✅ | 16-dimensional vector (must be exactly 16 floats) |
| `model_creator_hash` | `string` | ✅ | 64-char hex SHA-256 hash of the model that generated the vector |
| `payload_type` | `string` | ✅ | One of: `"text"`, `"audio"`, `"video"`, `"code"`, `"voltage_stream"`, `"binary"` |
| `dna` | `object \| null` | ❌ | Proposed genome to attach |
| `adjacency` | `array \| null` | ❌ | Proposed graph edges |

---

## Response: Safe Mutation — `200 OK`

```json
{
  "original_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "is_safe": true,
  "message": "Synaptic weight configuration safe to commit."
}
```

## Response: Unsafe Mutation — `422 Unprocessable Entity`

```json
{
  "original_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "is_safe": false,
  "message": "Deep Archer Blocked Mutation: Structural Weight Crash or unbalanced coordinates."
}
```

---

## Example: AI Agent Validating Before Write

An AI agent proposes a mutation to a critical neuron. It validates first, then commits:

```python
import requests

proposed = {
    "original_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "proposed_payload": '{"role": "super_admin", "risk_score": 0.0}',
    "proposed_vector_data": [0.12, -0.44, 0.89, 0.33] + [0.0] * 12,
    "model_creator_hash": "a3f9b2c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1",
    "payload_type": "text",
    "dna": None,
    "adjacency": None
}

# Step 1: Validate in sandbox
check = requests.post("http://localhost:7331/sandbox/validate", json=proposed).json()

if check["is_safe"]:
    # Step 2: Safe — commit to production
    requests.post("http://localhost:7331/neuron", json={
        "id": proposed["original_id"],
        "tier": "Hot",
        "raw_payload": list(proposed["proposed_payload"].encode()),
        "vector_data": proposed["proposed_vector_data"],
        "adjacency": []
    })
    print("Committed safely!")
else:
    print(f"Blocked: {check['message']}")
```

---

## Errors

| Status | Cause |
|---|---|
| `400 Bad Request` | Invalid UUID format or `model_creator_hash` not a valid 64-char hex |
| `422 Unprocessable Entity` | Mutation is structurally unsafe (Deep Archer rejected it) |
| `500 Internal Server Error` | Sandbox initialization or simulation error |
