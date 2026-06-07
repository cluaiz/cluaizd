# CRISPR Surgery API

> *"When a neuron goes rogue, don't delete it — clamp it."*

## What is the CRISPR API?

CRISPR is CNSDB's **surgical precision tool** for live database intervention. Named after the gene-editing technology, it allows you to:

1. **Clamp** a DNA parameter to a forced value without restarting the server.
2. **Force-inject** a permanent synaptic edge that cannot be removed by normal write operations.

These are emergency operations — use them for critical interventions, not routine data management.

---

## `POST /crispr/clamp/{id}` — Lock a DNA Parameter

Forces a specific `parameters` field inside a Neuron's attached Genome DNA to a fixed value. Useful for emergency risk management (e.g., hard-locking a fraud risk coefficient to 1.0 for a flagged account).

**Headers:**
- `x-tenant-id`: (optional) Shard name. Defaults to `default_sandbox`.
- `Content-Type: application/json`

**Request Body:**
```json
{
  "key": "risk_coefficient",
  "value": 1.0
}
```

**Example:**
```bash
# Hard-lock the risk_coefficient of a flagged user account to 1.0 (maximum risk)
curl -X POST http://localhost:7331/crispr/clamp/a1b2c3d4-e5f6-7890-abcd-ef1234567890 \
  -H "Content-Type: application/json" \
  -H "x-tenant-id: user_accounts" \
  -d '{ "key": "risk_coefficient", "value": 1.0 }'
```

**Response:**
```json
{
  "status": "success",
  "message": "Clamped parameter 'risk_coefficient' to 1"
}
```

**Errors:**
- `400 Bad Request` — Invalid UUID format, or Neuron has no DNA attached.
- `404 Not Found` — Neuron ID does not exist in the specified shard.

---

## `POST /crispr/force/{id}` — Inject a Permanent Graph Edge

Injects a synaptic edge into a Neuron's adjacency list. Unlike normal adjacency (which can be overwritten on the next `POST /neuron`), this edge is written directly to LMDB and also updates the JUJU spatial map.

Use this for: emergency relationship corrections, authority node connections (a "root" node that must always point to specific children), fraud network linking.

**Headers:**
- `x-tenant-id`: (optional) Shard name.
- `Content-Type: application/json`

**Request Body:**
```json
{
  "target_id": "target-neuron-uuid-here",
  "weight": 0.95
}
```

**Example:**
```bash
# Force-link two accounts detected as part of the same fraud ring
curl -X POST http://localhost:7331/crispr/force/a1b2c3d4-e5f6-7890-abcd-ef1234567890 \
  -H "Content-Type: application/json" \
  -H "x-tenant-id: default_sandbox" \
  -d '{ "target_id": "f0e1d2c3-b4a5-9876-dcba-ef9876543210", "weight": 0.95 }'
```

**Response:**
```json
{
  "status": "success",
  "message": "Forced synaptic edge to target 'f0e1d2c3-b4a5-9876-dcba-ef9876543210'"
}
```

**Notes:**
- Duplicate edge injection is idempotent — if the edge already exists, the call is a no-op.
- The JUJU spatial map is updated immediately after a successful force-edge.

---

## When to Use CRISPR vs Normal Writes

| Scenario | Use This |
|---|---|
| Normal data insert | `POST /neuron` |
| Update a neuron's payload | `POST /neuron` (same ID) |
| Emergency: lock a DNA parameter | `POST /crispr/clamp/{id}` |
| Emergency: force a permanent graph link | `POST /crispr/force/{id}` |
| Schema enforcement | Genome `on_write` hook |

> [!WARNING]
> CRISPR operations bypass genome `on_write` validation hooks. They write directly to LMDB. Use with caution in production.
