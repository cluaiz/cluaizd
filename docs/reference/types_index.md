# 📦 Data Types Reference (A-Z)

Welcome to the **cluaizd Data Types Reference**.
Understanding how the Engine serializes and stores specific data structures in the LMDB memory map is crucial for optimizing your schemas and maximizing query performance.

## Types Index

| Type | Category | Description |
|---|---|---|
| [`json`](./types/json.md) | Structured | Schemaless, deeply nested JSON objects mapped internally via binary representation.<hr>`{"role": "admin", "age": 25}` |
| [`vector`](./types/vector.md) | AI/ML | High-dimensional arrays of 32-bit floats (`f32`), optimized for SIMD distances.<hr>`[0.1, -0.9, 0.45]` |
