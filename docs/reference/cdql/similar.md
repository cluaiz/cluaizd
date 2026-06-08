# `similar` Command Reference

The `similar` keyword switches the cluaizd query executor from standard exact-match logic (JSON/Text) to High-Dimensional Vector Similarity Search. It is designed specifically for AI workflows, RAG (Retrieval-Augmented Generation), and semantic search.

## Syntax Rules

```text
find similar using <ALGORITHM> [ limit <K> ]
```

### Supported Algorithms
- `cosine_distance` : Best for normalized vectors (e.g., OpenAI embeddings).
- `euclidean_distance` : Best for spatial data and non-normalized vectors.

---

## Architecture: How it works under the hood

Executing a vector search is highly computationally intensive compared to a standard B-Tree lookup.

### 1. SIMD Acceleration
When a `find similar` query is executed, the engine isolates the `vector_data` arrays stored inside the target records. It leverages hardware-level SIMD (Single Instruction, Multiple Data) intrinsics (like AVX-512 or NEON, depending on the CPU architecture) to compute the mathematical distance between the client's `query_vector` and millions of stored vectors in parallel.

### 2. Top-K Heap Sort
Because similarity searches require comparing the query against the entire dataset to find the "closest" matches, the engine maintains an internal Min-Heap of size `K` (where `K` is the `limit` parameter). As the engine streams through the LMDB memory map, it only retains the top `K` results in memory, ensuring that RAM usage remains constant $O(K)$ regardless of the database size.

---

## Time Complexity

| Operation                  | Complexity             | Notes                                                                                                                                                                          |
| :------------------------- | :--------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Brute Force Similarity** | **O(N * d + N log K)** | `N` is the total records, `d` is the vector dimensions, and `K` is the limit. Currently, cluaizd relies on extreme SIMD throughput rather than HNSW indices for exact accuracy. |

---

## Examples

### 1. Semantic Search using Cosine Distance

This is the standard approach when using embeddings from LLM providers like OpenAI or Cohere.

```json
{
  "cdql": "find similar using cosine_distance limit 5",
  "query_vector": [0.015, -0.022, 0.991, 0.401, -0.112]
}
```
