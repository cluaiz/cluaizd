# 🚚 Schema & Data Migration Tests

## 🎯 Purpose
This workspace ensures that database upgrades and schema migrations do not cause data loss.

## 🧬 What we test here
1. **Forward Migrations:** Does the engine successfully upgrade an older LMDB shard format?
2. **Rollbacks:** If a migration fails midway, is the database restored safely?

## 🚀 How to Run
```bash
cargo test --package migration-tests
```
