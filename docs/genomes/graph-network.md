# Graph Network Genome (`graph_network.json`)

> *"Data is not a table. It's a web."*

## When to Use This Genome
Use the `graph_network` genome when:
- Relationships between entities are as important as the entities themselves.
- You need to traverse multi-hop connections (friends of friends, causal chains).
- You are building social networks, recommendation engines, knowledge graphs, or fraud detection.
- You need to find the shortest path between two nodes.

Real-world use cases: LinkedIn connections, product recommendations, fraud ring detection, supply chain tracking, AI reasoning chains, road navigation.

---

## The Core Concept: Index-Free Adjacency

Relational DB has foreign keys. When you JOIN two tables, the database performs a hash lookup across both tables' indexes — this is O(log n) per hop.

Graph DB (and CLUAIZD in graph mode) use a completely different strategy called **Index-Free Adjacency**. Every Neuron stores a direct list of its neighbor's IDs (the `adjacency` field). When traversing an edge, CLUAIZD follows a direct memory pointer instead of performing an index lookup.

This changes graph traversal from **O(n × log n)** to **O(edges)** — a dramatic speedup as graphs grow.

### The `adjacency` Field
Every `UniversalNeuron` stores its edges here:
```json
"adjacency": [
  { "target_id": "user_bob", "relation": "friends", "weight": 0.9 },
  { "target_id": "user_carol", "relation": "follows", "weight": 0.5 },
  { "target_id": "post_001", "relation": "authored", "weight": 1.0 }
]
```
- `target_id`: The neighboring Neuron's ID.
- `relation`: The edge type (like an SQL foreign key name, but richer).
- `weight`: Optional edge strength (0.0 - 1.0). Used for ranking traversal results.

---

## Building a Social Graph

### Insert Users with Connections
```bash
# Insert User A
curl -X POST http://localhost:7331/data -d '{
  "id": "user_alice",
  "tier": "Hot",
  "raw_payload": [bytes for {"name": "Alice"}],
  "vector_data": [],
  "adjacency": [
    { "target_id": "user_bob", "relation": "friends", "weight": 1.0 },
    { "target_id": "user_carol", "relation": "friends", "weight": 0.8 }
  ]
}'

# Insert User B
curl -X POST http://localhost:7331/data -d '{
  "id": "user_bob",
  "tier": "Hot",
  "raw_payload": [bytes for {"name": "Bob"}],
  "vector_data": [],
  "adjacency": [
    { "target_id": "user_dave", "relation": "friends", "weight": 0.7 }
  ]
}'
```

---

## Variable-Depth Traversals

### Find Friends (1 hop)
```text
find id("user_alice") -> traverse(edge: "friends", hops: 1..1)
```

### Find Friends of Friends (2 hops)
```text
find id("user_alice") -> traverse(edge: "friends", hops: 1..2)
```

### Find Anyone Reachable in 5 Hops
```text
find id("user_alice") -> traverse(edge: "friends", hops: 1..5)
```

### Filter by Edge Weight (High-Quality Connections Only)
```text
// Only traverse edges with weight >= 0.8 (strong connections)
find id("user_alice") -> traverse(edge: "friends", min_weight: 0.8, hops: 1..3)
```

---

## Fraud Ring Detection (Advanced Use Case)

Graph databases are the industry standard for detecting fraudulent account rings. A fraud ring is a set of accounts that appear independent but share hidden connections (same IP, same device, same bank account).

```text
// Find all accounts that share a device fingerprint with a known fraudster
find Account(id: "fraud_account_X")
  -> traverse(edge: "shares_device", hops: 1..3)
  -> filter risk_score > 0.7
  -> sort_by("risk_score", asc: false)
  -> limit 50
```

---

## AI Reasoning Chains

Store an AI agent's causal reasoning as graph edges:

```bash
# The AI stores: "Inflation → Rate Hike → Tech Layoffs → Churn Risk"
curl -X POST http://localhost:7331/data -d '{
  "id": "concept_inflation",
  "tier": "Hot",
  "raw_payload": [bytes for {"concept": "Inflation Rising"}],
  "adjacency": [
    { "target_id": "concept_rate_hike", "relation": "causes", "weight": 0.92 }
  ]
}'
```

Later, the AI retrieves its reasoning:
```text
find id("concept_inflation") -> traverse(edge: "causes", hops: 1..5)
```

---

## Comparison: CLUAIZD vs Graph DB

| Feature | Graph DB | CLUAIZD (graph_network) |
|---|---|---|
| Index-Free Adjacency | ✅ | ✅ |
| Variable-Depth Traversal | ✅ | ✅ |
| Shortest Path (Dijkstra) | ✅ | 🔜 (Planned) |
| Built-in Cypher QL | ✅ | ✅ (via CDQL) |
| Vector Search on Nodes | ❌ | ✅ (switch genome) |
| Zero-setup (no JVM) | ❌ | ✅ |
