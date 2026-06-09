# 📚 Cluaizd Documentation

Welcome to the official Cluaizd documentation. This folder contains everything you need to understand, run, and scale Cluaizd.

---

## 🧭 Table of Contents

### 1. 🌟 Vision & Overview
Understand why Cluaizd exists and the philosophy behind its architecture.
- [Why Cluaizd?](vision/why-cluaizd.md)
- [For AI Agents](vision/for-ai-agents.md)

### 2. ⚡ Getting Started
Get up and running locally in under 60 seconds.
- [Quickstart](get-started/quickstart.md)
- [Installation Guide](get-started/installation.md)
- [Your First Mutating DB](get-started/your_first_mutating_db.md)

### 3. 🏗️ Architecture Deep Dive
Learn the internals of how Cluaizd achieves 0ms latency and supports multiple database models.
- [LMDB Zero-Copy Storage](architecture/lmdb-zero-copy.md)
- [FFI Bypass (0ms Latency)](architecture/ffi-bypass.md)
- [Sensory Shards (High-Frequency)](architecture/sensory-shards.md)
- [The Dreamer Engine (GC & Analytics)](architecture/dreamer-engine.md)
- [Concurrency & Serialization Engine](architecture/concurrency-serialization.md)

### 4. 🗃️ 30 Database Paradigms (Engine Taxonomy)
Cluaizd operates as a multi-model database engine, supporting 30 distinct paradigms native to the `UniversalNeuron` core:
- **Index & Deep Dive Guides**: Check out the comprehensive markdown guides located directly inside [docs/cluaizd-types/](cluaizd-types/):
  - [01-key-value.md](cluaizd-types/01-key-value.md) | [02-document-nosql.md](cluaizd-types/02-document-nosql.md) | [03-relational-oltp.md](cluaizd-types/03-relational-oltp.md)
  - [04-columnar-olap.md](cluaizd-types/04-columnar-olap.md) | [05-vector-similarity.md](cluaizd-types/05-vector-similarity.md) | [06-labeled-property-graph.md](cluaizd-types/06-labeled-property-graph.md)
  - [07-rdf-triple-store.md](cluaizd-types/07-rdf-triple-store.md) | [08-time-series.md](cluaizd-types/08-time-series.md) | [09-spatial-gis.md](cluaizd-types/09-spatial-gis.md)
  - [10-blob-object.md](cluaizd-types/10-blob-object.md) | [11-inverted-index.md](cluaizd-types/11-inverted-index.md) | [12-hierarchical.md](cluaizd-types/12-hierarchical.md)
  - [13-network-model.md](cluaizd-types/13-network-model.md) | [14-event-store.md](cluaizd-types/14-event-store.md) | [15-pub-sub.md](cluaizd-types/15-pub-sub.md)
  - [16-blockchain-ledger.md](cluaizd-types/16-blockchain-ledger.md) | [17-ledger-blockchain.md](cluaizd-types/17-ledger-blockchain.md) | [18-content-addressable.md](cluaizd-types/18-content-addressable.md)
  - [19-grid-data.md](cluaizd-types/19-grid-data.md) | [20-multi-valued.md](cluaizd-types/20-multi-valued.md) | [21-probabilistic.md](cluaizd-types/21-probabilistic.md)
  - [22-crdt.md](cluaizd-types/22-crdt.md) | [23-operational-transformation.md](cluaizd-types/23-operational-transformation.md) | [24-array-tensor.md](cluaizd-types/24-array-tensor.md)
  - [25-semantic-web.md](cluaizd-types/25-semantic-web.md) | [26-multivalued.md](cluaizd-types/26-multivalued.md) | [27-probabilistic-sketches.md](cluaizd-types/27-probabilistic-sketches.md)
  - [28-crdt-collaborative.md](cluaizd-types/28-crdt-collaborative.md) | [29-analytical-cube.md](cluaizd-types/29-analytical-cube.md) | [30-embedded-in-process.md](cluaizd-types/30-embedded-in-process.md)

### 5. 🧬 DNA Rulesets & Serialization Templates
See executable rulesets, schema files, and performance blueprints in the [DNA Templates Hub](cluaizd-dna-templates/README.md):
- **Dynamic Rhai Scripts**: [CRUD Schema](cluaizd-dna-templates/crud_schema.rhai) | [Weight Decay](cluaizd-dna-templates/weight_decay.rhai) | [Time Series Decay](cluaizd-dna-templates/time_series_decay.rhai) | [Hybrid Vector-Tag Search](cluaizd-dna-templates/hybrid_search.rhai)
- **Zero-Copy WASM**: [Rust-WASM DNA Engine](cluaizd-dna-templates/wasm_example.rs) | [Auto-WASM Schema Verification](cluaizd-dna-templates/auto_wasm_example.rs)
- **Declarative Frameworks**: [JSON Rule Schema](cluaizd-dna-templates/json_rules.json) | [CDQL Ingestion Pipeline](cluaizd-dna-templates/cdql_example.md)
- **Binary Serialization**: [FlatBuffers & Protobuf Guide (Zero-Copy & Companion Object Avoidance)](cluaizd-dna-templates/serialization_formats.md)

### 6. 🧬 The 4 Execution Engines (DNA)
Explore the 4 runtime modes (CDQL, WASM, Auto-WASM, Rhai) and the built-in paradigms.
- [DNA Architecture & The 4 Engines](genomes/dna-architecture.md)
- [Auto-WASM Strict Types Guide](genomes/auto-wasm-guide.md)
- [Key-Value DNA](genomes/key-value.md)
- [Graph Network DNA](genomes/graph-network.md)
- [Vector & AI DNA](genomes/vector-ai.md)
- [Document NoSQL DNA](genomes/document-nosql.md)

### 7. ⚡ CDQL (Cluaiz Database Query Language)
Master the universal query language pipeline.
- [CDQL Intro](query-language/cdql-intro.md)
- [Syntax Reference](query-language/syntax-reference.md)
- [Advanced Pipelines](cdql/advanced-pipelines.md)
- [Rosetta Stone Cheatsheet](cdql/rosetta-stone.md)

### 8. 📡 Clients & APIs
How to connect to Cluaizd from your application.
- [REST API Reference](clients/rest-api.md)
- [C-FFI Bindings](clients/c-ffi.md)

---

*Note: All documentation is fully indexed and searchable via `registry.json`.*
