# `ctx.fetch` Engine API

The `ctx.fetch()` instruction allows a WASM or Rhai script to pause its current execution flow, request another independent record from the database by its UUID, and bring that secondary record's payload into the sandbox.

## Architectural Execution

### 1. Pointer Dereferencing vs B-Tree Lookups
If the requested UUID is linked to the current record via a Graph Edge (created via the `/crispr/force` API), `ctx.fetch()` executes a pure pointer dereference. The engine jumps directly to the physical memory offset, resulting in extreme sub-microsecond latency. If there is no Graph Edge, the engine performs a standard `O(log N)` B-Tree index lookup.

### 2. Transaction Scope
Any record retrieved via `ctx.fetch()` is bound to the current database transaction. This ensures strict ACID consistency—if the secondary record is currently locked by a concurrent write, the WASM thread yields (via Tokio async runtimes) and awaits the lock release, preventing phantom reads.

## Time Complexity

| Operation                  | Complexity   | Notes                                                                      |
| :------------------------- | :----------- | :------------------------------------------------------------------------- |
| **Fetch via Graph Edge**   | **O(1)**     | Constant time due to direct memory pointer jumping.                        |
| **Fetch via B-Tree Index** | **O(log N)** | Logarithmic time to traverse the secondary index if no direct edge exists. |

## Example: Relational Enrichment

```rust
fn on_read(ctx) {
    let mut payload = ctx.get_payload_json();
    
    // Check if the record has a parent company ID
    if let Some(company_id) = payload.get("company_id") {
        
        // Fetch the company record directly into the WASM sandbox
        if let Some(company_record) = ctx.fetch(company_id) {
            
            // Enrich the current payload with the company name before returning it to the client
            payload.insert("company_name", company_record.get("name"));
        }
    }
    
    ctx.set_ephemeral_json(payload);
}
```
