# 📦 SDK Bindings Tests

## 🎯 Purpose
This workspace ensures that all external SDKs (Rust, Python, Node.js bindings) interact perfectly with the core database engine.

## 🧬 What we test here
1. **Client Library Mechanics:** Do the SDK helper functions correctly encode/decode binary payloads?
2. **Network Resilience:** Can the SDK handle sudden connection drops or retries gracefully?

## 🚀 How to Run
```bash
cargo test --package sdk-tests
```
