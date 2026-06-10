# 🎲 Fuzzing & Randomized Inputs

## 🎯 Purpose
This workspace focuses on `cargo-fuzz` style randomized testing. It bombards the database's CDQL parser and DNA engines with random, malformed bytes to discover obscure panic-inducing edge cases.

## 🚀 How to Run
```bash
cargo test --package fuzz-tests
```
