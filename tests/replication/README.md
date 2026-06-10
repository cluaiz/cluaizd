# 🌐 Distributed Replication Tests

## 🎯 Purpose
This workspace focuses on distributed systems testing for multi-node setups (future clusters).

## 🧬 What we test here
1. **Consistency Checkers:** Linearizability tests when multiple shards sync.
2. **Network Partitions:** Split-brain scenario testing.

## 🚀 How to Run
```bash
cargo test --package replication-tests
```
