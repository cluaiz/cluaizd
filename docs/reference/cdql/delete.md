# `delete` Command Reference

The `delete` command permanently removes records from the database storage engine.

## Syntax Rules

```text
delete where [ CONDITION ]
```

---

## Architecture: How it works under the hood

Deleting records in a high-performance engine requires handling dangling references and reclaiming disk space without stalling the system.

### 1. Tombstone Marking
When a `delete` query executes, the engine does not immediately zero out the bytes on the physical disk. Instead, the record's B-Tree node is marked with a "Tombstone". Any concurrent or future `find` queries will see the tombstone and skip the record.

### 2. Graph Edge Pruning
If the deleted record had outgoing or incoming Graph Edges (synaptic connections created via the `force` API), those edges are automatically orphaned. The background Telemetry engine eventually prunes these dead edges during idle CPU cycles.

### 3. Memory Compaction
The LMDB engine reclaims the tombstoned memory pages. These pages are added to a "freelist" and will be reused for future `insert` operations, ensuring the database file does not bloat infinitely.

---

## Time Complexity

| Operation            | Complexity   | Notes                                                                            |
| :------------------- | :----------- | :------------------------------------------------------------------------------- |
| **Indexed Delete**   | **O(log N)** | Instantaneous tombstone placement via B-Tree lookup.                             |
| **Unindexed Delete** | **O(N)**     | Requires a full sequential scan to identify records matching the `where` clause. |

---

## Examples

### 1. Soft Threshold Deletion

Deleting all cache records that have surpassed their Time-to-Live (TTL).

```json
{
  "cdql": "delete where expires_at < 1690000000"
}
```

### 2. Exact Record Deletion

Removing a specific record by its UUID.

```json
{
  "cdql": "delete where id == 'uuid-999'"
}
```
