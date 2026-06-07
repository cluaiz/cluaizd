/// Base Vector DNA — CLUAIZD WASM Module
///
/// Enables high-dimensional cosine similarity search.
/// Compares a 16-dimensional query vector against the neuron's stored vector_data.
///
/// Query format (JSON):
/// ```json
/// {"vector": [0.1, 0.2, ...16 floats...], "threshold": 0.7}
/// ```
/// threshold: minimum cosine similarity to be considered a match (0.0 to 1.0)
///
/// Payload format: the neuron's vector_data as a JSON array:
/// ```json
/// [0.1, 0.2, ...16 floats...]
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

/// Execute vector similarity query.
/// Returns 1 if cosine similarity >= threshold, 0 otherwise.
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

    // Extract query vector and threshold
    let query_vec: Vec<f32> = match query.get("vector").and_then(|v| v.as_array()) {
        Some(arr) => arr.iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect(),
        None => return 0,
    };

    let threshold = query.get("threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.7) as f32;

    // Parse the stored payload vector (the neuron's vector_data)
    let stored_vec: Vec<f32> = match serde_json::from_slice::<Vec<f32>>(payload_bytes) {
        Ok(v) => v,
        Err(_) => {
            // Try parsing as a JSON object with a "vector" field
            if let Ok(obj) = serde_json::from_slice::<serde_json::Value>(payload_bytes) {
                match obj.get("vector").and_then(|v| v.as_array()) {
                    Some(arr) => arr.iter()
                        .filter_map(|v| v.as_f64().map(|f| f as f32))
                        .collect(),
                    None => return 0,
                }
            } else {
                return 0;
            }
        }
    };

    if query_vec.len() != stored_vec.len() || query_vec.is_empty() {
        return 0;
    }

    // Compute cosine similarity
    let dot: f32 = query_vec.iter().zip(stored_vec.iter()).map(|(a, b)| a * b).sum();
    let mag_a: f32 = query_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = stored_vec.iter().map(|x| x * x).sum::<f32>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0;
    }

    let similarity = dot / (mag_a * mag_b);

    if similarity >= threshold { 1 } else { 0 }
}

/// Compute cosine similarity score as a scaled integer (0–1000).
/// Called separately from execute_query to get the actual score.
#[no_mangle]
pub extern "C" fn similarity_score(
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

    let query_vec: Vec<f32> = match query.get("vector").and_then(|v| v.as_array()) {
        Some(arr) => arr.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect(),
        None => return 0,
    };

    let stored_vec: Vec<f32> = match serde_json::from_slice::<Vec<f32>>(payload_bytes) {
        Ok(v) => v,
        Err(_) => return 0,
    };

    if query_vec.len() != stored_vec.len() || query_vec.is_empty() {
        return 0;
    }

    let dot: f32 = query_vec.iter().zip(stored_vec.iter()).map(|(a, b)| a * b).sum();
    let mag_a: f32 = query_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = stored_vec.iter().map(|x| x * x).sum::<f32>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0;
    }

    ((dot / (mag_a * mag_b)) * 1000.0) as i32
}
