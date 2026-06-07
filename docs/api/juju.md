# JUJU Spatial Canvas API

> *"See your database like a living brain — every neuron, every synapse, in real-time 3D."*

## What is JUJU?

JUJU is CLUAIZD's **Live Spatial State Engine**. While every other database shows you a list of rows, JUJU maps every Neuron and its graph edges onto a real-time **3D coordinate space**.

This enables the **Genome Canvas** (the GUI) to render your entire database as a living neural network — where each node's position reflects its tier (Hot/Warm/Cold) and each line represents a real graph edge in your data.

---

## How the Spatial Map is Built

Every time a neuron is written or an edge is created (via `POST /neuron` or `POST /crispr/force/{id}`), the server updates an in-memory `SpatialMap`. The JUJU endpoint serves this map on demand.

```rust
pub struct SpatialCoordinates {
    pub x: f32,  // X position in the canvas
    pub y: f32,  // Y position in the canvas
    pub z: f32,  // Z position (depth = tier: 0=Hot, 1=Warm, 2=Cold)
    pub tier: String,  // "Hot" | "Warm" | "Cold"
}

pub struct SpatialMap {
    pub nodes: HashMap<String, SpatialCoordinates>,  // NeuronID → coordinates
    pub edges: HashMap<String, Vec<String>>,          // NeuronID → [target IDs]
}
```

---

## `GET /juju/state` — Fetch the Live Spatial Map

Returns the current snapshot of all neurons and their graph edges in the 3D coordinate space.

**Request:**
```bash
curl http://localhost:7331/juju/state
```

**Response:**
```json
{
  "nodes": {
    "a1b2c3d4-...": { "x": 120.5, "y": 340.2, "z": 0.0, "tier": "Hot" },
    "e5f6g7h8-...": { "x": 580.1, "y": 210.8, "z": 0.0, "tier": "Hot" },
    "i9j0k1l2-...": { "x": 340.0, "y": 480.0, "z": 1.0, "tier": "Warm" }
  },
  "edges": {
    "a1b2c3d4-...": ["e5f6g7h8-...", "i9j0k1l2-..."],
    "e5f6g7h8-...": ["i9j0k1l2-..."]
  }
}
```

---

## Integration with the Genome Canvas UI

The Genome Canvas GUI (`apps/gui`) polls `GET /juju/state` every 500ms and renders the `SpatialMap` using a 3D force-directed graph visualization. 

Node Z-position encodes the storage tier:
- `z: 0.0` = **Hot** tier (bright, active)
- `z: 1.0` = **Warm** tier (dimmed, fading)
- `z: 2.0` = **Cold** tier (deep, compressed, gray)

---

## Use Cases

- **Debugging graph data:** Visually verify that edge traversals are connected correctly.
- **Performance monitoring:** Watch neurons migrate from Hot → Warm → Cold as the Dreamer runs.
- **Data validation:** Instantly see if isolated nodes (no edges) exist when they should be connected.
