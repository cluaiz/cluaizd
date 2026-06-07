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

/// Executes the query. Returns 1 for match, 0 for no match.
/// In a real implementation, this would parse a query like `{"age": 25}`
/// and check if `payload_bytes` contains `{"name": "Aryan", "age": 25}`.
#[no_mangle]
pub extern "C" fn execute_query(
    query_ptr: *const u8,
    query_len: usize,
    payload_ptr: *const u8,
    payload_len: usize,
) -> i32 {
    let query_slice = unsafe { slice::from_raw_parts(query_ptr, query_len) };
    let payload_slice = unsafe { slice::from_raw_parts(payload_ptr, payload_len) };

    // For now, let's implement a very simple JSON parsing using serde_json.
    // If query is a sub-object of payload, we return 1.
    
    // Fallback: If both fail to parse, just check if query is a substring of payload bytes.
    // This is a naive implementation just to prove the pipeline.
    
    let query_val: Result<serde_json::Value, _> = serde_json::from_slice(query_slice);
    let payload_val: Result<serde_json::Value, _> = serde_json::from_slice(payload_slice);
    
    if let (Ok(q), Ok(p)) = (query_val, payload_val) {
        if let (Some(q_obj), Some(p_obj)) = (q.as_object(), p.as_object()) {
            // Check if all key-value pairs in query exist in payload
            for (k, v) in q_obj {
                if p_obj.get(k) != Some(v) {
                    return 0; // Mismatch
                }
            }
            return 1; // Full match
        }
    }
    
    // Fallback naive byte search
    let query_str = String::from_utf8_lossy(query_slice);
    let payload_str = String::from_utf8_lossy(payload_slice);
    
    if payload_str.contains(query_str.as_ref()) {
        1
    } else {
        0
    }
}
