# Relational SQL Genome (`sql_strict.json`)

> *"All the power of PostgreSQL, without a single line of SQL DDL."*

## When to Use This Genome
Use the `sql_strict` genome when your data requires:
- **Strict schemas** — Every record must have specific fields. Missing fields must abort the write.
- **ACID-like validation** — Business rules that prevent corrupt or incomplete records from entering the system.
- **Relational operations** — Joining data from different Neurons by shared key fields.

Real-world use cases: User accounts, billing records, inventory management, financial transactions.

---

## How It Works (Under the Hood)

### Schema Enforcement via `on_write`
Unlike PostgreSQL where the schema is compiled into the engine's C code, CNSDB enforces schemas entirely through the `on_write` Rhai hook in the Genome script. When a Neuron write is requested, the core engine **calls this hook before committing to LMDB**.

If the hook returns `"Abort"`, the transaction is rolled back atomically. No partial writes. No dirty state.

### The Default `sql_strict.json` Genome

Located at `genomes/sql_strict.json`:
```json
{
  "on_write": "let res = #{ action: \"Allow\" };\nif type_of(payload) != \"map\" {\n    res.action = \"Abort\";\n    res.error = \"ACID Violation: Payload must be a strictly typed map (table row)\";\n} else {\n    if !payload.contains(\"id\") || !payload.contains(\"created_at\") {\n        res.action = \"Abort\";\n        res.error = \"Schema Violation: Missing required columns 'id' or 'created_at'\";\n    }\n}\nres",
  "on_read": "let res = #{ payload: payload };\nres",
  "on_index": null,
  "on_lifecycle": null,
  "parameters": {},
  "engine": "rhai"
}
```

---

## Writing Your Own Strict Schema

You are not limited to the default `sql_strict.json`. You can define custom per-table schemas by writing a new Genome.

### Example: A `products` Table Schema
```json
{
  "on_write": "
    let res = #{ action: 'Allow' };
    let required = ['id', 'name', 'price_usd', 'stock_count', 'category'];
    
    for field in required {
      if !payload.contains(field) {
        res.action = 'Abort';
        res.error = `Missing required field: ${field}`;
        return res;
      }
    }
    
    if payload.price_usd < 0 {
      res.action = 'Abort';
      res.error = 'price_usd cannot be negative';
    }
    
    if payload.stock_count < 0 {
      res.action = 'Abort';
      res.error = 'stock_count cannot be negative';
    }
    
    res
  ",
  "engine": "rhai"
}
```

This genome enforces a schema with 5 required fields and two business rules — exactly like a PostgreSQL `CHECK CONSTRAINT`.

---

## Relational Joins with CNQL

CNSDB does not have foreign keys in the SQL sense. Relationships are expressed through Graph **edges** (`adjacency` field). However, CNQL's `join()` operator allows you to merge payload data from related Neurons at query time.

### Example: Join Orders with Products
```text
// Find all orders, join the related product's name and price
find Order(status: "pending")
  -> join(target: "Product", on: "product_id == target.id", type: "inner")
  -> filter target.price_usd > 50
  -> limit 100
```

This is equivalent to:
```sql
SELECT o.*, p.name, p.price_usd 
FROM orders o
INNER JOIN products p ON o.product_id = p.id
WHERE p.price_usd > 50
LIMIT 100;
```

---

## Group By and Aggregations

```text
// Count total orders per user, sum their total spend
find Order(status: "completed")
  -> group_by("user_id")
  -> aggregate(count(), sum(price_usd))
  -> sort_by("count", asc: false)
  -> limit 10
```

This returns the top 10 users by order count — a standard analytics query.

---

## Best Practices

> [!TIP]
> **Index your most-queried fields.** While CNSDB does not have traditional B-tree indexes, you can store a pre-computed sorted list of IDs in a dedicated "Index Neuron" and traverse it via graph edges for fast lookups.

> [!WARNING]
> **Do not use `sql_strict` for high-frequency IoT streams.** The Rhai hook evaluation adds ~0.1ms per write. For 10,000 writes/second, use the `sensory_stream` genome instead which has a lightweight append-only validation.
