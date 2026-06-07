# Supported Paradigms (The 10-in-1 Engine)

By writing the correct DNA, CNSDB can morph into 10 entirely different database paradigms. Below are the official, out-of-the-box Genomes supported by CNSDB.

> [!NOTE]
> All these genomes exist in the `genomes/` directory. You can load them at runtime without recompiling the Rust server.

## 1. Relational SQL (`sql_strict.json`)
Behaves like PostgreSQL. 
- **Enforces:** Strict payload typing (must be a JSON map).
- **Enforces:** Required fields (e.g., `id`, `created_at`). If missing, the transaction aborts.

## 2. Document NoSQL (`document_store.json`)
Behaves like MongoDB.
- **Enforces:** Nothing. Accepts raw BSON/JSON payloads and allows dynamic schema mutation.

## 3. Time-Series (`time_series.json`)
Behaves like InfluxDB.
- **Enforces:** Requires a `timestamp` field on insertion.
- **Lifecycle:** Automatically downsamples old granular data into heavily compressed `Cold` tier chunks to save disk space.

## 4. Graph Network (`graph_network.json`)
Behaves like Neo4j.
- **Index-Free Adjacency:** Enforces memory pointers between nodes. When you run `traverse()`, it jumps pointers in `O(1)` time without doing heavy Hash-Joins.

## 5. Key-Value (`ephemeral_cache.json`)
Behaves like Redis / Memcached.
- **Fast-Path:** When queried via ID (`find id("x")`), CNSDB bypasses the WASM engine entirely, reading directly from LMDB memory-maps for `0ms` latency.
- **Lifecycle:** strict LRU / TTL eviction policies.

## 6. Vector / AI (`vector_space.json`)
Behaves like Pinecone / Milvus.
- Stores dense Float32 embeddings. 
- WASM engine executes Cosine/L2 distance calculations for `similar_to()` queries.

## 7. Geo-Spatial (`geospatial.json`)
Behaves like PostGIS.
- Automatically calculates Haversine distances for `geo_near` queries based on attached Lat/Lon metadata coordinates.

## 8. Wide-Column (`sensory_stream.json`)
Behaves like Cassandra.
- **Append-Only:** Mutating existing rows is blocked. Built for massive throughput IoT and BCI sensory streams.

## 9. Full-Text Search (`search_index.json`)
Behaves like Elasticsearch.
- Asynchronously builds an Inverted Index on write. Allows fuzzy typo-tolerant queries via BM25 scoring.

## 10. Blob Storage (`object_store.json`)
Behaves like S3.
- Bypasses RAM constraints by forcing `Cold` tier ZSTD compression for massive Video/Audio chunks, enabling Byte-Range streaming.
