# `search` — Full-Text Search Reference

The `search` pipeline stage executes a **full-text / inverted-index query** against every neuron's JSON payload in the working set.
It tokenises both the query and each payload, computes a match score, and returns only records that contain at least one matching token — sorted from highest to lowest relevance.

---

## Syntax

```text
find * -> search(query: "<TERMS>", fuzzy: <bool>)
```

| Parameter | Type     | Required | Description |
| :-------- | :------- | :------- | :---------- |
| `query`   | `string` | ✅ Yes   | One or more space-separated search terms. |
| `fuzzy`   | `bool`   | ✅ Yes   | `true` → substring match. `false` → exact whole-word match. |

---

## Architecture: How It Works Under the Hood

The `search` step is evaluated inside [`crates/genome/src/cdql/eval.rs`](../../../../crates/genome/src/cdql/eval.rs) by the `eval_full_text()` function.

### Execution Pipeline

```
CDQL string
   │
   ▼
 parse()        ← crates/genome/src/cdql/parser.rs
   │  Produces CdqlQuery AST
   ▼
 build_plan()   ← crates/genome/src/cdql/planner.rs
   │  Emits PlanStep::FullTextSearch { query, fuzzy }
   ▼
 execute_cdql() ← crates/server/src/routes/query.rs
   │  Iterates every neuron in the working set
   ▼
 eval_full_text(payload, query, fuzzy)
   │  ← crates/genome/src/cdql/eval.rs
   │
   ├── 1. extract_text_from_json(payload)
   │       Recursively walks JSON tree, concatenates all string-leaf values.
   │       Falls back to raw bytes if payload is not valid JSON.
   │
   ├── 2. Tokenise query  → ["database", "engine"]
   │   Tokenise payload → ["a", "high", "performance", "database", "engine", ...]
   │
   ├── 3. Per token: exact match (fuzzy=false) OR substring match (fuzzy=true)
   │
   └── 4. score = matched_tokens / total_query_tokens   ∈ [0.0, 1.0]
              score == 0.0 → neuron is excluded
              score  > 0.0 → neuron is included
   │
   ▼
 Results sorted descending by score (highest relevance first)
```

### Text Extraction from JSON

The engine does **not** require you to put your searchable text in a specific field.
It recursively collects every `string` and `number` leaf in the JSON tree:

```json
{
  "title":  "Cluaiz Database Engine",
  "meta": {
    "author": "Aryan",
    "version": 3
  }
}
```
Searchable text extracted → `"Cluaiz Database Engine Aryan 3"`

### Fuzzy vs Exact

| Mode | `fuzzy` | Match condition | Example |
| :--- | :------ | :-------------- | :------ |
| Exact | `false` | Query token **==** payload word (case-insensitive) | `"data"` does NOT match `"database"` |
| Fuzzy | `true`  | Payload word **contains** query token (case-insensitive) | `"data"` DOES match `"database"` |

---

## Scoring & Ranking

Every neuron that survives receives a score:

```
score = (number of query tokens found in payload) / (total query tokens)
```

| Query tokens | Found in payload | Score |
| :----------- | :--------------- | :---- |
| `["database", "engine"]` | both found | `1.0` |
| `["database", "engine"]` | only `"database"` | `0.5` |
| `["database", "engine"]` | none found | `0.0` → **excluded** |

Results are returned sorted **descending by score**. No additional `-> sort` step is needed.

---

## Time Complexity

| Scenario | Complexity | Notes |
| :------- | :--------- | :---- |
| Unindexed full scan | **O(N × T × W)** | N = neurons, T = query tokens, W = average payload word count |
| After `-> limit K` | **O(N × T × W)** | Limit is applied *after* scoring — full scan still occurs |

> **Performance Tip**: Use `-> limit` after `-> search` to cap result set size and reduce network serialisation cost.

---

## Examples

### 1. Exact Multi-Word Search

```json
{
  "tenant_id": "my_tenant",
  "cdql": "find * -> search(query: \"database engine\", fuzzy: false)"
}
```
Returns all neurons whose payload contains **both** the word `database` **and** the word `engine`, sorted by how many terms matched.

---

### 2. Fuzzy Single-Term Search

```json
{
  "tenant_id": "my_tenant",
  "cdql": "find * -> search(query: \"rust\", fuzzy: true)"
}
```
Returns any neuron whose payload contains a word that **includes** `rust` — e.g., `"rustacean"`, `"rusty"`, `"rust"`.

---

### 3. Chained with `limit`

```json
{
  "tenant_id": "my_tenant",
  "cdql": "find * -> search(query: \"machine learning\", fuzzy: false) -> limit 10"
}
```
Returns the top 10 most relevant neurons about machine learning.

---

### 4. Chained with `range` (Compound Query)

```json
{
  "tenant_id": "my_tenant",
  "cdql": "find * -> search(query: \"sensor telemetry\", fuzzy: true) -> range(field: \"severity\", start: 3, end: 5)"
}
```
First filters neurons by keyword relevance, then applies a severity range scan on the survivors.

---

## API Request Shape

`POST /query`

```json
{
  "tenant_id": "my_tenant",
  "cdql": "find * -> search(query: \"<TERMS>\", fuzzy: <true|false>)"
}
```

### Response

```json
[
  {
    "score": 1.0,
    "neuron": {
      "id": "uuid-...",
      "raw_payload": { "description": "A high-performance database engine written in Rust" }
    }
  },
  {
    "score": 0.5,
    "neuron": {
      "id": "uuid-...",
      "raw_payload": { "description": "A distributed database for real-time analytics" }
    }
  }
]
```

---

## Error Handling

| Condition | Behaviour |
| :-------- | :-------- |
| `query` is empty string | All neurons score `0.0` → empty result set returned |
| Payload is not valid JSON | Entire raw bytes treated as one searchable string |
| Payload field contains non-string (number, bool) | Converted to string representation and included in search |

---

## Related

- [`range`](./range.md) — Numeric or lexicographic range filtering
- [`geo_near`](./geo_near.md) — Geo-spatial proximity filtering
- [`find`](./find.md) — Base traversal command
- [`limit`](./limit.md) — Result count truncation
- [CDQL Advanced Pipelines](../../cdql/advanced-pipelines.md) — Chaining multiple pipeline stages
