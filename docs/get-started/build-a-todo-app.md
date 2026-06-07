# Build Your First App: A Real-Time Todo Board

In this tutorial, we will build a **real-time collaborative todo app** backend using CLUAIZD. Unlike a typical "Hello World" tutorial, we will use CLUAIZD as 3 different databases simultaneously:
- **Relational (SQL-like):** For structured Todo items with strict validation.
- **Real-Time Cache (Redis-like):** For storing online user presence with TTL.
- **Graph (Neo4j-like):** For linking todos to users via ownership edges.

This is CLUAIZD's core superpower on full display.

---

## Step 0: Prerequisites
Make sure CLUAIZD is running on `localhost:7331`. See the [Installation Guide](installation.md).

---

## Step 1: Create a "Users" Neuron (Relational Mode)

We will use the `sql_strict` genome to enforce that every user must have an `id` and `created_at` field.

```bash
# Insert User 1 — Aryan
curl -X POST http://localhost:7331/data \
  -H "Content-Type: application/json" \
  -d '{
    "id": "user_aryan",
    "tier": "Hot",
    "raw_payload": [123,34,110,97,109,101,34,58,34,65,114,121,97,110,34,125],
    "vector_data": [],
    "adjacency": [],
    "dna": {
      "engine": "rhai",
      "on_write": "let res = #{ action: \"Allow\" }; if !payload.contains(\"name\") { res.action = \"Abort\"; res.error = \"name is required\"; } res",
      "parameters": {}
    }
  }'
```

The `on_write` hook will reject any write that does not contain the `name` field. Try submitting without it and observe the `Abort` error.

---

## Step 2: Create a Todo Item (Relational Mode)

```bash
curl -X POST http://localhost:7331/data \
  -H "Content-Type: application/json" \
  -d '{
    "id": "todo_001",
    "tier": "Hot",
    "raw_payload": [123,34,116,105,116,108,101,34,58,34,66,117,121,32,109,105,108,107,34,44,34,100,111,110,101,34,58,102,97,108,115,101,125],
    "vector_data": [],
    "adjacency": [
      { "target_id": "user_aryan", "relation": "owned_by", "weight": 1.0 }
    ]
  }'
```

Notice the `adjacency` field — we just created a graph edge from `todo_001` to `user_aryan` with relation `owned_by`. CLUAIZD is simultaneously acting as a Relational DB (for structure) and a Graph DB (for edges).

---

## Step 3: Store Online Presence with TTL (Cache Mode)

Now let's track which users are online using the `ephemeral_cache` genome with a 5-minute TTL:

```bash
curl -X POST http://localhost:7331/data \
  -H "Content-Type: application/json" \
  -d '{
    "id": "presence_aryan",
    "tier": "Hot",
    "raw_payload": [123,34,115,116,97,116,117,115,34,58,34,111,110,108,105,110,101,34,125],
    "vector_data": [],
    "adjacency": [],
    "dna": {
      "engine": "rhai",
      "on_lifecycle": "let res = #{}; if age_ns > 300000000000 { res.action = \"Evict\"; } res",
      "parameters": { "ttl_ns": 300000000000 }
    }
  }'
```

After 5 minutes, the Dreamer thread will automatically evict `presence_aryan` from memory. No cron job required.

---

## Step 4: Query All Todos for a User (Graph + Filter)

```bash
curl -X POST http://localhost:7331/query \
  -H "Content-Type: application/json" \
  -d '{
    "cnql": "find * -> filter done: false -> limit 50"
  }'
```

For a graph-aware query to get all todos owned by Aryan:
```bash
curl -X POST http://localhost:7331/query \
  -H "Content-Type: application/json" \
  -d '{
    "cnql": "find id(\"user_aryan\") -> traverse(edge: \"owned_by\")"
  }'
```

---

## Step 5: Mark a Todo as Done (Update)

```bash
curl -X PUT http://localhost:7331/data/todo_001 \
  -H "Content-Type: application/json" \
  -d '{
    "raw_payload": [123,34,116,105,116,108,101,34,58,34,66,117,121,32,109,105,108,107,34,44,34,100,111,110,101,34,58,116,114,117,101,125]
  }'
```

---

## What We Just Built

In this single tutorial, we used CLUAIZD as:
1. **PostgreSQL** — Strict schema enforcement via `on_write` Rhai hooks.
2. **Neo4j** — Graph edges (`owned_by`) linking Todos to Users.
3. **Redis** — TTL-based presence cache with auto-eviction.

All from one HTTP API. Zero additional databases. Zero infrastructure sprawl.

Next: Learn how each Genome works in depth → [The 10 Genomes](../genomes/relational-sql.md).
