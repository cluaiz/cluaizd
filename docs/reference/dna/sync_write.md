# `sync_write` Durability Control

The `sync_write` setting in a Genome allows you to dynamically control the data durability and flush strategy on a *per-record* basis. This means you can have a single database where critical financial transactions are strictly synced to the hardware, while sensor telemetry is written with maximum lock-free throughput.

## Configuration Options

When defining a Genome, you can set `sync_write` to one of two modes (either as a string or a boolean):

### 1. Paranoia Mode (`"strict"` or `true`)
- **What it does**: Bypasses the OS page cache and forces a direct, synchronous hardware-level `.sync_all()` to the SSD block device. 
- **Data Safety**: 100% Guaranteed. Even if the server loses power exactly 1 microsecond after the HTTP response is sent, the data is safe on the disk.
- **Performance**: Slower. Bound by your SSD's IOPS limit (typically 1,000 - 5,000 writes per second).
- **Use Case**: Financial ledgers, user passwords, highly critical mutations.

### 2. Lightning Mode (`"lite"` or `false`)
- **What it does**: Writes the mutation to the OS Page Cache (RAM) and immediately returns success to the client. The OS will lazily flush the buffer to the physical disk in the background.
- **Data Safety**: 99.99%. If the *application* crashes, the data is safe (OS handles it). If the *entire physical server loses power* before the flush, recent milliseconds of data may be lost.
- **Performance**: Extremely Fast. Can easily exceed 20,000+ OPS since it avoids blocking on physical disk spin/flash.
- **Use Case**: Analytics, sensory data, live gaming metrics, AI embeddings generation.

---

## Example Genome Configuration

```json
{
  "engine": "cdql",
  "sync_write": "lite",
  "on_write": "let res = #{ action: \"Allow\" }; res",
  "parameters": {}
}
```

> [!TIP]
> If you omit `sync_write` from your Genome JSON, CLUAIZD will fall back to `"lite"` as the default for maximum performance. If you require absolute ACID durability, always explicitly set `"sync_write": "strict"`.
