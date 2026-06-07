# Document NoSQL Genome (`document_store.json`)

> *"Store anything. Shape it later."*

## When to Use This Genome
Use the `document_nosql` genome when:
- Your data schema evolves rapidly and cannot be pre-defined.
- You store heterogeneous records where each document may have different fields.
- You need deep nested JSON structures with arrays and sub-objects.
- You are building a CMS, social network, or any product-driven app.

Real-world use cases: Blog posts, user profiles, product catalogs, chat messages, event logs.

---

## How It Works

Unlike `sql_strict`, the `document_store` genome applies zero schema validation. It accepts any valid JSON payload and stores it as-is. The `on_write` hook only ensures the payload is parseable JSON.

### The `document_store.json` Genome
```json
{
  "on_write": "let res = #{ action: \"Allow\" };\nif type_of(payload) == \"()\" {\n    res.action = \"Abort\";\n    res.error = \"Payload cannot be null\";\n}\nres",
  "on_read": "let res = #{ payload: payload };\nres",
  "engine": "rhai"
}
```

---

## Storing Complex Nested Documents

```bash
# Store a blog post with nested tags array and author sub-object
curl -X POST http://localhost:7331/data \
  -H "Content-Type: application/json" \
  -d '{
    "id": "post_001",
    "tier": "Hot",
    "raw_payload": [... bytes of JSON below ...],
    "vector_data": [],
    "adjacency": []
  }'
```

The raw_payload represents:
```json
{
  "title": "Why CLUAIZD Changes Everything",
  "content": "Full article text here...",
  "author": {
    "id": "user_aryan",
    "name": "Aryan",
    "role": "admin"
  },
  "tags": ["database", "rust", "ai", "neural"],
  "published": true,
  "views": 0,
  "created_at": "2026-06-07T22:00:00Z"
}
```

No schema was defined. No `CREATE TABLE`. No migration. It just works.

---

## Querying Nested Fields

CNQL can filter on top-level fields. For nested fields, use dot notation:

```text
// Find all published posts
find Post(published: true) -> limit 20

// Find posts with more than 1000 views
find Post(published: true) -> filter views > 1000

// Sort by views descending
find Post(published: true) -> sort_by("views", asc: false) -> limit 10
```

---

## Schema-on-Read: The Projection Pipeline

Even without a defined schema, you can shape the output at query time using `project()`:

```text
// Return only title, author.name, and tags — strip everything else
find Post(published: true)
  -> project(keep: ["title", "author", "tags"])
  -> limit 50
```

---

## Array Unwinding (MongoDB `$unwind` Equivalent)

If a document has a `tags` array and you want to analyze each tag individually:

```text
// Unwind the tags array, then filter for only "rust" tagged posts
find Post(published: true)
  -> unwind("tags")
  -> filter tags: "rust"
  -> limit 20
```

---

## Best Practices

> [!NOTE]
> Even though `document_store` is schema-free, consider writing a lightweight `on_write` validation in Rhai for your most critical fields (e.g., `title` must be present). This catches bugs early without the overhead of `sql_strict`.

> [!TIP]
> Combine `document_store` with the `search_index` genome for posts. The `on_index` hook in `search_index.json` will automatically extract the `title` and `content` fields into an Inverted Index, enabling fuzzy full-text search across all blog posts.
