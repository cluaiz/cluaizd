# 📚 Cluaizd Documentation

Welcome to the official Cluaizd documentation. This folder contains everything you need to understand, run, and scale Cluaizd.

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
Learn the internals of how Cluaizd achieves 0ms latency and supports 10 paradigms.
- [LMDB Zero-Copy Storage](architecture/lmdb-zero-copy.md)
- [FFI Bypass (0ms Latency)](architecture/ffi-bypass.md)
- [Sensory Shards (High-Frequency)](architecture/sensory-shards.md)
- [The Dreamer Engine (GC & Analytics)](architecture/dreamer-engine.md)
- [Concurrency & Serialization Engine](architecture/concurrency-serialization.md)

### 4. 🧬 The 4 Execution Engines (DNA)
Explore the 4 runtime modes (CDQL, WASM, Auto-WASM, Rhai) and the built-in paradigms.
- [DNA Architecture & The 4 Engines](genomes/dna-architecture.md)
- [Auto-WASM Strict Types Guide](genomes/auto-wasm-guide.md)
- [Key-Value DNA](genomes/key-value.md)
- [Graph Network DNA](genomes/graph-network.md)
- [Vector & AI DNA](genomes/vector-ai.md)
- [Document NoSQL DNA](genomes/document-nosql.md)
- *See the `genomes/` folder for the full list of supported DNAs.*

### 5. ⚡ CDQL (Cluaiz Database Query Language)
Master the universal query language pipeline.
- [CDQL Intro](query-language/cdql-intro.md)
- [Syntax Reference](query-language/syntax-reference.md)
- [Advanced Pipelines](cdql/advanced-pipelines.md)
- [Rosetta Stone Cheatsheet](cdql/rosetta-stone.md)

### 6. 📡 Clients & APIs
How to connect to Cluaizd from your application.
- [REST API Reference](clients/rest-api.md)
- [C-FFI Bindings](clients/c-ffi.md)

---

*Note: All documentation is fully indexed and searchable via `registry.json`.*
