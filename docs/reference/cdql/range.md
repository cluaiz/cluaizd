# `range` — Range Scan Reference

The `range` pipeline stage executes a **field-level boundary scan** against the working set.
It reads a named field from each neuron's JSON payload and retains only those neurons whose value falls within the specified `[start, end]` interval (inclusive on both ends).

Supports two comparison modes:
- **Numeric** — IEEE-754 f64 boundary comparison
- **Lexicographic** — UTF-8 string ordering (dictionary order)

---

## Syntax

```text
find * -> range(field: "<KEY>", start: <VALUE>, end: <VALUE>)
```

| Parameter | Type            | Required | Description |
| :-------- | :-------------- | :------- | :---------- |
| `field`   | `string`        | ✅ Yes   | The JSON key to read from each neuron's payload. |
| `start`   | `number\|string` | ✅ Yes   | Lower bound of the range (inclusive). |
| `end`     | `number\|string` | ✅ Yes   | Upper bound of the range (inclusive). |

> The types of `start` and `end` must match. Mixing `number` and `string` produces no results.

---

## Architecture: How It Works Under the Hood

The `range` step is evaluated inside [`crates/genome/src/cdql/eval.rs`](../../../../crates/genome/src/cdql/eval.rs) by the `eval_range()` function.

### Execution Pipeline

```
CDQL string
   │
   ▼
 parse()        ← crates/genome/src/cdql/parser.rs
   │  Produces CdqlQuery AST with CdqlValue::Number or CdqlValue::Text boundaries
   ▼
 build_plan()   ← crates/genome/src/cdql/planner.rs
   │  Emits PlanStep::RangeScan { field, start, end }
   ▼
 execute_cdql() ← crates/server/src/routes/query.rs
   │  Filters the working set via eval_range()
   ▼
 eval_range(payload, field, &start, &end)
   │  ← crates/genome/src/cdql/eval.rs
   │
   ├── 1. serde_json::from_str(payload)
   │       Parse neuron payload as JSON object.
   │       Returns false on parse failure — never panics.
   │
   ├── 2. json.get(field)
   │       Extract the target key from the JSON map.
   │       Returns false if key is absent.
   │
   ├── 3a. Numeric mode (start & end are Number):
   │       neuron_value.as_f64()
   │       Passes if:  start ≤ value ≤ end
   │
   └── 3b. String mode (start & end are Text):
           neuron_value.as_str()
           Passes if:  start ≤ value ≤ end  (UTF-8 lexicographic)
```

### Type Safety

The comparison mode is determined entirely by the types of `start` and `end` in the CDQL query string — the engine performs **zero implicit type coercion**:

| `start` type | `end` type | Comparison mode |
| :----------- | :--------- | :-------------- |
| `number`     | `number`   | IEEE-754 f64 numeric |
| `string`     | `string`   | UTF-8 lexicographic |
| `number`     | `string`   | ❌ No match — returns `false` |

---

## Boundary Semantics

Both `start` and `end` are **inclusive**:

```
value ∈ [start, end]  →  included
value < start          →  excluded
value > end            →  excluded
```

---

## Time Complexity

| Scenario | Complexity | Notes |
| :------- | :--------- | :---- |
| Unindexed scan | **O(N)** | Sequentially reads every neuron's payload |
| No match (field absent) | **O(N)** | Still scans all records; field probe fails per neuron |

> **Performance Tip**: Place `-> range(...)` **before** expensive pipeline stages like `-> search(...)` to reduce the working set early.

---

## Examples

### 1. Numeric Range — Age Filter

```json
{
  "tenant_id": "my_tenant",
  "cdql": "find * -> range(field: \"age\", start: 18, end: 35)"
}
```
Returns all neurons where `payload.age` is between 18 and 35 (inclusive).

---

### 2. Numeric Range — Salary Band

```json
{
  "tenant_id": "my_tenant",
  "cdql": "find * -> range(field: \"salary\", start: 60000, end: 120000)"
}
```

---

### 3. Lexicographic Range — City Alphabetical Slice

```json
{
  "tenant_id": "my_tenant",
  "cdql": "find * -> range(field: \"city\", start: \"Chennai\", end: \"Pune\")"
}
```
Returns neurons where `payload.city` falls between `"Chennai"` and `"Pune"` in alphabetical (UTF-8) order.
Includes: `"Delhi"`, `"Hyderabad"`, `"Mumbai"`.
Excludes: `"Ahmedabad"` (before `"Chennai"`), `"Surat"` (after `"Pune"`).

---

### 4. Chained with `search` (Compound Query)

```json
{
  "tenant_id": "my_tenant",
  "cdql": "find * -> range(field: \"price\", start: 100, end: 500) -> search(query: \"wireless\", fuzzy: true)"
}
```
Finds products priced between ₹100 and ₹500 that are also described as "wireless".

---

### 5. Chained with `geo_near` (Compound Spatial Query)

```json
{
  "tenant_id": "my_tenant",
  "cdql": "find * -> geo_near(lat: 28.6139, lon: 77.2090, radius_km: 50.0) -> range(field: \"rating\", start: 4, end: 5)"
}
```
Finds locations within 50 km of New Delhi that also have a rating of 4–5 stars.

---

## API Request Shape

`POST /query`

```json
{
  "tenant_id": "my_tenant",
  "cdql": "find * -> range(field: \"<KEY>\", start: <VALUE>, end: <VALUE>)"
}
```

### Response

```json
[
  {
    "neuron": {
      "id": "uuid-...",
      "raw_payload": { "name": "Alice", "age": 25, "salary": 70000 }
    }
  },
  {
    "neuron": {
      "id": "uuid-...",
      "raw_payload": { "name": "Carol", "age": 28, "salary": 85000 }
    }
  }
]
```

> Range results are not scored — ordering matches the underlying LMDB iteration order unless further sorted by a downstream pipeline stage.

---

## Error Handling

| Condition | Behaviour |
| :-------- | :-------- |
| Field key is absent from payload | Neuron is **excluded** silently |
| Payload is not valid JSON | Neuron is **excluded** silently — no panic |
| `start` > `end` | No error raised; simply zero neurons match (empty result) |
| Mixed types (`number` start, `string` end) | Neuron is **excluded** silently — type guard returns `false` |

---

## Related

- [`search`](./search.md) — Full-text keyword search
- [`geo_near`](./geo_near.md) — Geo-spatial proximity filtering
- [`gt`](./gt.md) — Single-bound greater-than filter
- [`lt`](./lt.md) — Single-bound less-than filter
- [`where`](./where.md) — General equality/comparison filter
- [CDQL Advanced Pipelines](../../cdql/advanced-pipelines.md) — Chaining multiple pipeline stages
