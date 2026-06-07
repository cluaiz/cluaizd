# C-FFI Bindings: Direct Memory Access

> **Max throughput: ~1,000,000 writes/second**  
> **Latency: ~0.5-2µs per operation**  
> **Protocol: Shared memory (no TCP, no JSON parsing)**

The C-FFI completely bypasses the HTTP server, Axum router, and JSON serialization layers. It writes directly into LMDB's memory-mapped file from your process's address space.

> [!IMPORTANT]
> For a full deep-dive into WHY the FFI is faster and how to use it for BCI/Robotics use cases, see the [FFI Bypass Manual](../architecture/ffi-bypass.md). This page covers the core API reference.

---

## Step 1: Build the Library

```bash
# Compile as a dynamic shared library
cargo build --release -p cluaizd-ffi

# Output locations:
# Windows:  target/release/cluaizd.dll
# Linux:    target/release/libcluaizd.so
# macOS:    target/release/libcluaizd.dylib
```

Copy the output library and `ffi/cluaizd.h` into your project.

---

## Full API Reference (`cluaizd.h`)

```c
#include "cluaizd.h"

// ─────────────────────────────────────────────
// cluaizd_open()
// Open (or create) an LMDB shard at the given path.
//
// @path:        Filesystem path to the LMDB directory (created if missing)
// @map_size_mb: Maximum database size in MB (e.g. 8192 = 8 GB)
// @return:      Opaque handle, or NULL on failure
// ─────────────────────────────────────────────
CluaizdHandle* cluaizd_open(const char* path, unsigned long map_size_mb);

// ─────────────────────────────────────────────
// cluaizd_write()
// Write raw bytes into LMDB — NO genome hooks, NO HTTP, NO JSON parser.
// This is the absolute fastest write path.
//
// @handle:       Handle from cluaizd_open()
// @neuron_id:    String key (any unique identifier)
// @payload_json: JSON string payload
// @return:       0 on success, -1 on failure
// ─────────────────────────────────────────────
int cluaizd_write(CluaizdHandle* handle, const char* neuron_id, const char* payload_json);

// ─────────────────────────────────────────────
// cluaizd_read()
// Direct LMDB key lookup — O(1) Fast-Path.
//
// @handle:    Handle from cluaizd_open()
// @neuron_id: The key to look up
// @return:    Heap-allocated JSON string. CALLER MUST FREE with cluaizd_free_string().
// ─────────────────────────────────────────────
char* cluaizd_read(CluaizdHandle* handle, const char* neuron_id);

// ─────────────────────────────────────────────
// cluaizd_query()
// Execute a CNQL query string against this shard.
//
// @handle: Handle from cluaizd_open()
// @cnql:   CNQL query string
// @return: Heap-allocated JSON array string. CALLER MUST FREE with cluaizd_free_string().
// ─────────────────────────────────────────────
char* cluaizd_query(CluaizdHandle* handle, const char* cnql);

// ─────────────────────────────────────────────
// cluaizd_free_string()
// Free a string returned by cluaizd_read() or cluaizd_query().
// ALWAYS call this to prevent memory leaks.
// ─────────────────────────────────────────────
void cluaizd_free_string(char* ptr);

// ─────────────────────────────────────────────
// cluaizd_close()
// Flush all pending writes and close the LMDB environment.
// ─────────────────────────────────────────────
void cluaizd_close(CluaizdHandle* handle);
```

---

## C++ Example: Sensor Ingestion

```cpp
#include "cluaizd.h"
#include <cstdio>
#include <cstring>

int main() {
    // Open a dedicated sensory shard (isolated from main database)
    CluaizdHandle* handle = cluaizd_open("./data/sensory_tissue", 8192);
    if (!handle) {
        fprintf(stderr, "Failed to open CLUAIZD\n");
        return 1;
    }

    // Write 1,000,000 sensor readings
    for (int i = 0; i < 1000000; i++) {
        char id[64];
        char payload[256];
        snprintf(id, sizeof(id), "reading_%07d", i);
        snprintf(payload, sizeof(payload),
            "{\"sensor\": \"temp_01\", \"value\": %.2f, \"ts\": %lld}",
            20.0 + (i % 10) * 0.5, (long long)i * 1000000LL);

        if (cluaizd_write(handle, id, payload) != 0) {
            fprintf(stderr, "Write failed at index %d\n", i);
            break;
        }
    }

    // Direct key lookup — bypasses CNQL entirely
    char* result = cluaizd_read(handle, "reading_0000500");
    if (result) {
        printf("Record: %s\n", result);
        cluaizd_free_string(result);  // Always free!
    }

    // CNQL query via FFI
    char* query_result = cluaizd_query(handle,
        "find * -> filter sensor: \"temp_01\" -> limit 10");
    if (query_result) {
        printf("Query: %s\n", query_result);
        cluaizd_free_string(query_result);  // Always free!
    }

    cluaizd_close(handle);
    return 0;
}
```

**Compile:**
```bash
gcc -o sensor_app sensor_app.c -L./target/release -lcluaizd -Wl,-rpath,./target/release
```

---

## Python Integration via `ctypes`

```python
import ctypes
import json

# Load the shared library
cluaizd = ctypes.CDLL("./target/release/libcluaizd.so")

# Configure return types (CRITICAL — without this, Python will crash)
cluaizd.cluaizd_open.restype = ctypes.c_void_p
cluaizd.cluaizd_open.argtypes = [ctypes.c_char_p, ctypes.c_ulong]

cluaizd.cluaizd_write.restype = ctypes.c_int
cluaizd.cluaizd_write.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]

cluaizd.cluaizd_read.restype = ctypes.c_char_p
cluaizd.cluaizd_read.argtypes = [ctypes.c_void_p, ctypes.c_char_p]

cluaizd.cluaizd_query.restype = ctypes.c_char_p
cluaizd.cluaizd_query.argtypes = [ctypes.c_void_p, ctypes.c_char_p]

cluaizd.cluaizd_free_string.argtypes = [ctypes.c_char_p]
cluaizd.cluaizd_close.argtypes = [ctypes.c_void_p]

# Open database
handle = cluaizd.cluaizd_open(b"./data/sensory_tissue", 4096)

# Write data
payload = json.dumps({"electrode": 42, "voltage_uv": 0.42, "ts": 1717789200}).encode()
cluaizd.cluaizd_write(handle, b"bci_reading_001", payload)

# Read back (direct O(1) lookup)
result_bytes = cluaizd.cluaizd_read(handle, b"bci_reading_001")
if result_bytes:
    print(json.loads(result_bytes))

# Close
cluaizd.cluaizd_close(handle)
```

---

## Performance vs HTTP

| Method | Throughput | Latency | Overhead |
|---|---|---|---|
| HTTP REST (`/neuron`) | ~50K writes/s | ~100µs | TCP + JSON + Axum routing |
| **C-FFI (`cluaizd_write`)** | **~1M writes/s** | **~1µs** | **None — direct mmap** |
| C-FFI + Ring Buffer | ~2M writes/s | ~0.5µs | Amortized batching |

> [!CAUTION]
> `cluaizd_write()` bypasses all Genome DNA hooks (`on_write`, `on_index`, `on_lifecycle`). Use it only for high-throughput sensor/BCI streams where you own the schema at the application level.
