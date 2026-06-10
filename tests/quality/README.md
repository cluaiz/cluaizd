# 🛡️ Quality & Property-Based Testing

## 🎯 Purpose
This workspace ensures the internal algorithms and parsing mechanics are mathematically and logically sound. 

## 🧬 What we test here
1. **CDQL Fuzzing:** Passing random strings to the parser to ensure it safely errors rather than panicking.
2. **Property-Based Testing:** Verifying invariants (e.g. `insert() + delete() == original state`).
3. **Search Mechanics:** Deep testing of the `cdql_search_mechanics.rs` to validate that scoring engines (TF-IDF, Cosine Similarity) return correct math.

## 🚀 How to Run
```bash
cargo test --package quality-tests
```
