/// Base SQL DNA — CNSDB WASM Module
///
/// Enables SQL-style comparison queries on JSON payloads.
/// Supports: =, !=, >, <, >=, <=, LIKE (contains), AND (multi-field)
///
/// Query format (JSON):
/// ```json
/// {"field": "age", "op": ">", "value": 18}
/// ```
/// Or multi-condition AND:
/// ```json
/// [{"field": "age", "op": ">", "value": 18}, {"field": "name", "op": "=", "value": "Aryan"}]
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

/// Execute SQL-style query.
/// Returns 1 if payload matches, 0 if not.
#[no_mangle]
pub extern "C" fn execute_query(
    query_ptr: *const u8,
    query_len: usize,
    payload_ptr: *const u8,
    payload_len: usize,
) -> i32 {
    let query_bytes = unsafe { slice::from_raw_parts(query_ptr, query_len) };
    let payload_bytes = unsafe { slice::from_raw_parts(payload_ptr, payload_len) };

    let payload: serde_json::Value = match serde_json::from_slice(payload_bytes) {
        Ok(v) => v,
        Err(_) => return 0,
    };

    let query: serde_json::Value = match serde_json::from_slice(query_bytes) {
        Ok(v) => v,
        Err(_) => return 0,
    };

    // Support both single condition and array of conditions (AND)
    let conditions = if query.is_array() {
        query.as_array().unwrap().clone()
    } else {
        vec![query]
    };

    // ALL conditions must match (AND logic)
    for condition in &conditions {
        let field = match condition.get("field").and_then(|v| v.as_str()) {
            Some(f) => f,
            None => return 0,
        };
        let op = match condition.get("op").and_then(|v| v.as_str()) {
            Some(o) => o,
            None => return 0,
        };
        let query_val = match condition.get("value") {
            Some(v) => v,
            None => return 0,
        };

        let payload_val = match payload.get(field) {
            Some(v) => v,
            None => return 0, // field doesn't exist → no match
        };

        let matches = match op {
            "=" | "==" | "eq" => payload_val == query_val,
            "!=" | "ne" => payload_val != query_val,
            ">" | "gt" => {
                match (payload_val.as_f64(), query_val.as_f64()) {
                    (Some(a), Some(b)) => a > b,
                    _ => false,
                }
            }
            "<" | "lt" => {
                match (payload_val.as_f64(), query_val.as_f64()) {
                    (Some(a), Some(b)) => a < b,
                    _ => false,
                }
            }
            ">=" | "gte" => {
                match (payload_val.as_f64(), query_val.as_f64()) {
                    (Some(a), Some(b)) => a >= b,
                    _ => false,
                }
            }
            "<=" | "lte" => {
                match (payload_val.as_f64(), query_val.as_f64()) {
                    (Some(a), Some(b)) => a <= b,
                    _ => false,
                }
            }
            "LIKE" | "like" | "contains" => {
                match (payload_val.as_str(), query_val.as_str()) {
                    (Some(a), Some(b)) => a.to_lowercase().contains(&b.to_lowercase()),
                    _ => false,
                }
            }
            _ => false,
        };

        if !matches {
            return 0; // AND logic: any failure = no match
        }
    }

    1 // All conditions matched
}
