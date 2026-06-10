# 🏎️ Performance Benchmarks

## 🎯 Purpose
This workspace focuses purely on speed. It uses the `Criterion` framework to prevent performance regressions across commits.

## 🧬 What we test here
1. **P99 Read Latency:** Must remain < 100µs.
2. **P99 Write Latency:** Must remain < 500µs.
3. **Throughput:** How many QPS (Queries Per Second) the core engine can handle before experiencing lock contention.

## 🚀 How to Run
```bash
cargo bench --package benchmarks
```
