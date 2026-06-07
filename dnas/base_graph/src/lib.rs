/// Base Graph DNA — CLUAIZD WASM Module
///
/// Enables graph traversal queries on the Neuron's adjacency edges.
/// Inspects edges, filters by weight and relation, and returns match/no-match.
///
/// Query format (JSON):
/// ```json
/// {
///   "relation": "friends",       // optional: filter by edge label/relation name
///   "min_weight": 0.5,           // optional: minimum edge weight to follow
///   "max_depth": 2,              // optional: max hops (1 = direct edges only)
///   "target_id": "uuid-string"   // optional: check if specific target is reachable
/// }
/// ```
///
/// Payload format: the neuron's adjacency list as JSON:
/// ```json
/// [{"target_id": "uuid", "weight": 0.8, "label": "friends"}, ...]
/// ```
use std::alloc::{alloc, dealloc, Layout};
use std::slice;

#[no_mangle]
pub extern "C" fn allocate(size: usize) -> *mut u8 {
    let layout = Layout::from_size_align(size, 1).unwrap();
    unsafe { alloc(layout) }
}

#[no_mangle]
pub extern "C" fn deallocate(ptr: *mut u8, size: usize) {
    let layout = Layout::from_size_align(size, 1).unwrap();
    unsafe { dealloc(ptr, layout) }
}

/// Execute a graph traversal query.
/// Returns 1 if the neuron has qualifying edges, 0 if not.
#[no_mangle]
pub extern "C" fn execute_query(
    query_ptr: *const u8,
    query_len: usize,
    payload_ptr: *const u8,
    payload_len: usize,
) -> i32 {
    let query_bytes = unsafe { slice::from_raw_parts(query_ptr, query_len) };
    let payload_bytes = unsafe { slice::from_raw_parts(payload_ptr, payload_len) };

    let query: serde_json::Value = match serde_json::from_slice(query_bytes) {
        Ok(v) => v,
        Err(_) => return 0,
    };

    // Parse adjacency list from payload
    let edges: Vec<serde_json::Value> = match serde_json::from_slice::<serde_json::Value>(payload_bytes) {
        Ok(serde_json::Value::Array(arr)) => arr,
        Ok(obj) => {
            // Try extracting from an object with "adjacency" field
            match obj.get("adjacency").and_then(|v| v.as_array()) {
                Some(arr) => arr.clone(),
                None => return 0,
            }
        }
        Err(_) => return 0,
    };

    if edges.is_empty() {
        return 0;
    }

    let min_weight = query.get("min_weight")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;

    let target_id = query.get("target_id").and_then(|v| v.as_str());
    let relation = query.get("relation").and_then(|v| v.as_str());

    // Check each edge
    for edge in &edges {
        let edge_weight = edge.get("weight")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;

        // Weight filter
        if edge_weight < min_weight {
            continue;
        }

        // Target ID filter
        if let Some(tid) = target_id {
            let edge_target = edge.get("target_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if edge_target != tid {
                continue;
            }
        }

        // Relation/label filter
        if let Some(rel) = relation {
            let edge_label = edge.get("label")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if edge_label != rel {
                continue;
            }
        }

        // This edge passes all filters — match!
        return 1;
    }

    0 // No qualifying edges found
}

/// Count qualifying edges (useful for ranking graph nodes by connectivity).
/// Returns count as integer (0 = no match).
#[no_mangle]
pub extern "C" fn count_edges(
    query_ptr: *const u8,
    query_len: usize,
    payload_ptr: *const u8,
    payload_len: usize,
) -> i32 {
    let query_bytes = unsafe { slice::from_raw_parts(query_ptr, query_len) };
    let payload_bytes = unsafe { slice::from_raw_parts(payload_ptr, payload_len) };

    let query: serde_json::Value = serde_json::from_slice(query_bytes).unwrap_or(serde_json::json!({}));
    let edges: Vec<serde_json::Value> = match serde_json::from_slice::<serde_json::Value>(payload_bytes) {
        Ok(serde_json::Value::Array(arr)) => arr,
        _ => return 0,
    };

    let min_weight = query.get("min_weight")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;

    edges.iter()
        .filter(|e| {
            e.get("weight")
                .and_then(|v| v.as_f64())
                .map(|w| w as f32 >= min_weight)
                .unwrap_or(false)
        })
        .count() as i32
}
