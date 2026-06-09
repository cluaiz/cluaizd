# `code` Data Type

The `code` format is designed for storing text representations of programming scripts, source code, or AST buffers.

## Architectural Storage

### Text Layout and Compression
Stored as standard UTF-8 text in the LMDB database. When transitioning to the **Cold Storage Tier**, code payloads are highly compressed using ZSTD Level 9, typically achieving 4x–10x compression ratios.

## Use Cases
- Storing executable script templates inside neurons.
- Versioned source code repositories.
