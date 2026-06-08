# `group_by` Aggregation Reference

The `group_by` affordance executes multi-dimensional bucketing, allowing developers to categorize and aggregate records based on shared attributes, identical to SQL's `GROUP BY` clause but executed inside a sandboxed WASM memory space.

## Architectural Execution

### 1. Memory-Mapped HashMaps
Instead of instantiating thousands of individual arrays on the heap (which triggers aggressive Garbage Collection pauses), the `group_by` affordance utilizes a specialized, continuous-memory `HashMap`. As the engine streams through the database records, it hashes the target "grouping key" (e.g., `"department"`) using the ultra-fast `FxHash` algorithm. 

### 2. Pointer Bucketing
To maintain zero-copy principles, the engine does not copy the entire JSON payload into the HashMap buckets. Instead, it only pushes the 64-bit physical memory offset of the record into the bucket array. This keeps the RAM footprint incredibly small even when grouping millions of records.

## Time Complexity

| Operation | Complexity | Notes |
| :--- | :--- | :--- |
| **Bucketing Phase** | **O(N)** | Requires traversing the dataset and computing the `FxHash` for each key. |
| **Reduction Phase** | **O(B)** | Where `B` is the number of distinct buckets created. |

## Example: Executing a Group By

```rust
fn execute_query(ctx) {
    let records = ctx.find_json("where region == 'US-East'");
    
    // 1. Bucket the records by the 'department' field
    let buckets = dna.bucket_by(records, "department");
    
    // 2. We can now iterate over the buckets and apply localized math
    let mut results = ctx.create_json_map();
    
    for (department_name, bucket_records) in buckets {
        // Calculate the sum of salaries exclusively for this department
        let dept_payroll = dna.aggregate("sum", bucket_records, "salary");
        results.insert(department_name, dept_payroll);
    }
    
    // 3. Return the grouped payroll report
    ctx.return_result(results);
}
```
