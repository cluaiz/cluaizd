# Booster WASM Engine API

> *"Swap your database's intelligence at runtime. No restart. No downtime."*

## What is the Booster?

The Booster is CNSDB's **hot-swappable WASM compute engine**. It allows you to upload a compiled `.wasm` binary that replaces the default computation engine powering certain genomic operations — without restarting the server.

Think of it as a CPU upgrade you can do while the machine is running.

---

## Booster Modes

The Booster operates in 7 modes, selectable at runtime:

| Mode ID | Mode Name | Description |
|---|---|---|
| `0` | `eco` | Minimum resource usage. Throttles background WASM execution. Best for low-traffic periods. |
| `1` | `balanced` | Default. Balances WASM compute speed with RAM conservation. |
| `2` | `performance` | Maximizes WASM execution thread count. Heavier RAM usage. |
| `3` | `ultra` | Unlocks all CPU cores for WASM execution. Not recommended for shared servers. |
| `4` | `ultramaxboost` | Experimental maximum throughput. Disables all rate limiting. |
| `5` | `auto` | Dynamically selects mode based on `sysinfo` RAM and CPU pressure. |
| `6` | `custom` | Uses the uploaded custom `system_booster.wasm` binary for compute decisions. |

---

## `POST /booster/upload` — Upload a Custom WASM Binary

Replaces the active compute engine with a new custom `.wasm` module. Submitted as a multipart form.

**Request (multipart/form-data):**
```bash
curl -X POST http://localhost:7331/booster/upload \
  -F "booster_wasm=@./my_custom_booster.wasm"
```

**Response:**
```json
{
  "status": "success",
  "message": "Successfully loaded system_booster.wasm"
}
```

**Errors:**
- `400 Bad Request` — Missing `booster_wasm` field in the multipart form.
- `500 Internal Server Error` — Failed to write WASM to disk.

**Notes:**
- The uploaded WASM is saved to `data/system_booster.wasm`.
- To activate the custom WASM, call `POST /booster/mode/custom` after uploading.

---

## `POST /booster/mode/{mode}` — Switch Active Mode

Changes the active booster mode. Takes effect immediately with no restart.

**Example: Switch to Performance mode before a large data import:**
```bash
curl -X POST http://localhost:7331/booster/mode/performance
```

**Example: Switch to Eco mode at night:**
```bash
curl -X POST http://localhost:7331/booster/mode/eco
```

**Example: Activate your custom WASM engine:**
```bash
# Step 1: Upload your WASM
curl -X POST http://localhost:7331/booster/upload -F "booster_wasm=@./my_booster.wasm"

# Step 2: Activate it
curl -X POST http://localhost:7331/booster/mode/custom
```

**Response:**
```json
{
  "status": "success",
  "message": "Switched booster mode to performance"
}
```

**Error (invalid mode):**
```json
{
  "status": "error",
  "message": "Invalid mode: turbo"
}
```

---

## Practical Automation

Use the Booster API to auto-scale compute based on time:

```bash
#!/bin/bash
# Cron: Switch to eco at midnight, performance at 9am
HOUR=$(date +%H)

if [ "$HOUR" -eq "00" ]; then
  curl -s -X POST http://localhost:7331/booster/mode/eco
elif [ "$HOUR" -eq "09" ]; then
  curl -s -X POST http://localhost:7331/booster/mode/performance
fi
```
