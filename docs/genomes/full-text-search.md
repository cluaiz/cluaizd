# Full-Text Search Genome (`search_index.json`)

> *"Find what users mean, not just what they type."*

## When to Use This Genome
Use the `search_index` genome when:
- Users search with natural language queries (e.g., a search bar).
- You need typo tolerance (users make spelling mistakes).
- Relevance ranking matters (the most relevant result should appear first).
- You need to search across multiple text fields with different importance weights.

Real-world use cases: E-commerce product search, documentation search, blog search, job board listings, customer support ticket lookup.

---

## The Science Behind Full-Text Search

### Inverted Index
A standard database `LIKE '%pizza%'` scans every row. An inverted index inverts this: instead of "document → words", it stores "word → documents that contain it".

```
"pizza"   → [doc_001, doc_034, doc_219]
"burger"  → [doc_002, doc_004]
"espresso"→ [doc_001, doc_089]
```

Finding all documents with "pizza" is now an `O(1)` lookup.

### BM25 Relevance Scoring
Not all matches are equal. BM25 (Best Match 25) is the industry-standard relevance algorithm. It scores documents based on:
- **Term Frequency (TF):** How often does "pizza" appear in this document?
- **Inverse Document Frequency (IDF):** How rare is "pizza" across ALL documents? (Rare words are more meaningful.)
- **Document Length:** Shorter documents with the same term count score higher.

### Levenshtein Fuzzy Matching
Levenshtein Distance is the number of single-character edits (insert, delete, replace) needed to transform one string into another. "Pizza" and "Pizaz" have a Levenshtein distance of 1 — they are close enough to match with `fuzzy: true`.

---

## Enabling Full-Text Search on a Collection

The `search_index.json` genome's `on_index` hook runs asynchronously after every write, extracting text fields and building the inverted index:

```json
{
  "on_index": "let res = #{};\nres.text_fields = [\"title\", \"content\", \"tags\"];\nres.language = \"english\";\nres",
  "engine": "rhai"
}
```

---

## Basic Text Search

```text
// Search for documents containing "database"
find * -> search(query: "database", fuzzy: false) -> limit 20
```

### With Fuzzy Matching (Typo Tolerance)
```text
// "databse" → will still find "database" documents
find * -> search(query: "databse", fuzzy: true) -> limit 20
```

---

## Field-Boosted Search

Some fields matter more than others. A "database" match in a document's `title` is more relevant than in its `footer`:

```text
// Title matches count 3x more than body matches
find Article
  -> search(fields: {title: 3.0, content: 1.0, tags: 2.0}, query: "neural database")
  -> sort_by_score()
  -> limit 10
```

---

## Combining Full-Text with Filters

```text
// Search only within published articles in the "tech" category
find Article(published: true, category: "tech")
  -> search(query: "machine learning", fuzzy: true)
  -> sort_by_score()
  -> limit 10
```

---

## E-Commerce Product Search Example

```python
# User types "blak shoez" (typo)
response = requests.post("http://localhost:7331/query", json={
    "cnql": """
        find Product(in_stock: true)
          -> search(fields: {name: 3.0, description: 1.0}, query: "blak shoez", fuzzy: true)
          -> sort_by_score()
          -> limit 20
    """
})
# Returns products matching "black shoes" despite the typos
```

---

## Comparison: CNSDB vs Elasticsearch

| Feature | Elasticsearch | CNSDB (search_index) |
|---|---|---|
| Inverted Index | ✅ | ✅ (via on_index hook) |
| BM25 Relevance | ✅ | ✅ |
| Fuzzy Matching | ✅ | ✅ |
| Field Boosting | ✅ | ✅ |
| Vector/Semantic Search | ✅ (kNN) | ✅ (hybrid genome) |
| JVM / Heap Tuning | ✅ (required, painful) | ❌ (not needed) |
| Cost (100GB index) | ~$150/mo | ~$5/mo |
