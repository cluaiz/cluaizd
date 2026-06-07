//! # CNSDB FFI — C-Compatible Bindings
//!
//! Exposes CNSDB as a native C library for **0ms local access**.
//! No HTTP, no serialization overhead — direct in-process memory calls.
//!
//! ## Use Cases
//! - **Robotics:** ROS2 nodes, embedded controllers (C/C++)
//! - **Python AI:** `ctypes` or `cffi` for direct ML model data ingestion
//! - **Brain-Computer Interfaces:** Neuralink/OpenBCI C SDKs
//! - **Game Engines:** Unreal/Godot C++ integrations
//!
//! ## Compile
//! ```sh
//! cargo build --release -p cnsdb-ffi
//! # Output: target/release/cnsdb.dll (Windows) / cnsdb.so (Linux)
//! ```
//!
//! ## C Usage
//! ```c
//! #include "cnsdb.h"
//!
//! CnsdbHandle* db = cnsdb_open("./data/mydb", 4096);
//! cnsdb_write(db, "hello world", 11, "text");
//! cnsdb_close(db);
//! ```

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_ulong};
use std::sync::Mutex;

use bytes::Bytes;
use cnsdb_types::{PayloadType, UniversalNeuron};
use engine_lmdb::LmdbEnv;

/// Opaque handle representing an open CNSDB database instance.
/// The caller holds a raw pointer — they must call `cnsdb_close` to free it.
pub struct CnsdbHandle {
    env: Mutex<LmdbEnv>,
}

/// Open a CNSDB database at the given path.
///
/// # Arguments
/// - `path`: UTF-8 path to the database directory (will be created if absent)
/// - `map_size_mb`: Maximum database size in megabytes (e.g. 4096 = 4GB)
///
/// # Returns
/// A non-null `CnsdbHandle*` on success, or `NULL` on failure.
///
/// # Safety
/// Caller must free the returned pointer with `cnsdb_close()`.
#[no_mangle]
pub extern "C" fn cnsdb_open(path: *const c_char, map_size_mb: c_ulong) -> *mut CnsdbHandle {
    if path.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let map_size = (map_size_mb as usize) * 1024 * 1024;
    match LmdbEnv::open(std::path::Path::new(path_str), map_size) {
        Ok(env) => {
            let handle = Box::new(CnsdbHandle {
                env: Mutex::new(env),
            });
            Box::into_raw(handle)
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Write a raw payload into CNSDB.
///
/// # Arguments
/// - `handle`: A valid `CnsdbHandle*` from `cnsdb_open()`
/// - `payload`: Pointer to the raw byte payload
/// - `payload_len`: Length of the payload in bytes
/// - `payload_type`: Type string: "text", "audio", "video", "code", "voltage_stream", or "binary"
///
/// # Returns
/// A heap-allocated null-terminated string containing the assigned Neuron UUID.
/// The caller MUST free this with `cnsdb_free_string()`.
/// Returns `NULL` on failure.
///
/// # Safety
/// `payload` must point to a valid buffer of at least `payload_len` bytes.
#[no_mangle]
pub extern "C" fn cnsdb_write(
    handle: *mut CnsdbHandle,
    payload: *const u8,
    payload_len: usize,
    payload_type: *const c_char,
) -> *mut c_char {
    if handle.is_null() || payload.is_null() || payload_type.is_null() {
        return std::ptr::null_mut();
    }

    let bytes = unsafe {
        let slice = std::slice::from_raw_parts(payload, payload_len);
        Bytes::copy_from_slice(slice)
    };

    let ptype_str = unsafe { CStr::from_ptr(payload_type) }
        .to_str()
        .unwrap_or("binary");

    let ptype = match ptype_str {
        "text" => PayloadType::Text,
        "audio" => PayloadType::Audio,
        "video" => PayloadType::Video,
        "code" => PayloadType::Code,
        "voltage_stream" => PayloadType::VoltageStream,
        _ => PayloadType::Binary,
    };

    let neuron = UniversalNeuron::new(bytes, [0.0f32; 16], [0u8; 32], ptype);
    let neuron_id = neuron.id.to_string();

    let handle_ref = unsafe { &*handle };
    let env = match handle_ref.env.lock() {
        Ok(e) => e,
        Err(_) => return std::ptr::null_mut(),
    };

    match engine_lmdb::write_neuron(&env, &neuron) {
        Ok(_) => {
            CString::new(neuron_id)
                .map(|s| s.into_raw())
                .unwrap_or(std::ptr::null_mut())
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Read a neuron's raw payload from CNSDB by its UUID string.
///
/// # Arguments
/// - `handle`: A valid `CnsdbHandle*`
/// - `neuron_id`: A null-terminated UUID string (e.g. "550e8400-e29b-41d4-...")
/// - `out_len`: Pointer to a `ulong` that receives the payload byte length
///
/// # Returns
/// A heap-allocated byte buffer containing the raw payload.
/// The caller MUST free this with `cnsdb_free_bytes()`.
/// Returns `NULL` if the neuron is not found.
///
/// # Safety
/// `out_len` must be a valid pointer.
#[no_mangle]
pub extern "C" fn cnsdb_read(
    handle: *mut CnsdbHandle,
    neuron_id: *const c_char,
    out_len: *mut c_ulong,
) -> *mut u8 {
    if handle.is_null() || neuron_id.is_null() || out_len.is_null() {
        return std::ptr::null_mut();
    }

    let id_str = unsafe { CStr::from_ptr(neuron_id) }
        .to_str()
        .unwrap_or("");

    let uuid = match uuid::Uuid::parse_str(id_str) {
        Ok(u) => u,
        Err(_) => return std::ptr::null_mut(),
    };

    let nid = cnsdb_types::NeuronId::from_bytes(*uuid.as_bytes());

    let handle_ref = unsafe { &*handle };
    let env = match handle_ref.env.lock() {
        Ok(e) => e,
        Err(_) => return std::ptr::null_mut(),
    };

    match engine_lmdb::read_neuron(&env, nid, None) {
        Ok(neuron) => {
            let payload = neuron.raw_payload.to_vec();
            let len = payload.len();
            let mut boxed = payload.into_boxed_slice();
            let ptr = boxed.as_mut_ptr();
            std::mem::forget(boxed);
            unsafe { *out_len = len as c_ulong };
            ptr
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Query CNSDB using a CNQL string.
///
/// # Arguments
/// - `handle`: A valid `CnsdbHandle*`
/// - `cnql`: A null-terminated CNQL query string (e.g. `find *(name: "Aryan")`)
///
/// # Returns
/// A heap-allocated null-terminated JSON string containing the array of matched neuron IDs.
/// The caller MUST free this with `cnsdb_free_string()`.
/// Returns `NULL` on failure.
#[no_mangle]
pub extern "C" fn cnsdb_query(
    handle: *mut CnsdbHandle,
    cnql: *const c_char,
) -> *mut c_char {
    if handle.is_null() || cnql.is_null() {
        return std::ptr::null_mut();
    }

    let query_str = unsafe { CStr::from_ptr(cnql) }
        .to_str()
        .unwrap_or("");

    let handle_ref = unsafe { &*handle };
    let env = match handle_ref.env.lock() {
        Ok(e) => e,
        Err(_) => return std::ptr::null_mut(),
    };

    // Get all neurons and do a simple payload search
    // (Full CNQL execution requires the genome crate — this is a simplified FFI version)
    match engine_lmdb::iter_all_neurons(&env) {
        Ok(neurons) => {
            let ids: Vec<String> = neurons.iter()
                .filter(|n| {
                    let payload = String::from_utf8_lossy(&n.raw_payload);
                    payload.to_lowercase().contains(&query_str.to_lowercase())
                })
                .map(|n| n.id.to_string())
                .collect();

            let json = serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string());
            CString::new(json)
                .map(|s| s.into_raw())
                .unwrap_or(std::ptr::null_mut())
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Close a CNSDB handle and free all associated resources.
///
/// # Safety
/// `handle` must be a valid pointer from `cnsdb_open()`. After this call, the pointer is invalid.
#[no_mangle]
pub extern "C" fn cnsdb_close(handle: *mut CnsdbHandle) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle)) };
    }
}

/// Free a string returned by CNSDB FFI functions.
///
/// # Safety
/// `ptr` must be a pointer returned by `cnsdb_write()` or `cnsdb_query()`.
#[no_mangle]
pub extern "C" fn cnsdb_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe { drop(CString::from_raw(ptr)) };
    }
}

/// Free a byte buffer returned by `cnsdb_read()`.
///
/// # Arguments
/// - `ptr`: The pointer returned by `cnsdb_read()`
/// - `len`: The length previously written to `out_len`
///
/// # Safety
/// `ptr` and `len` must match what was returned by `cnsdb_read()`.
#[no_mangle]
pub extern "C" fn cnsdb_free_bytes(ptr: *mut u8, len: c_ulong) {
    if !ptr.is_null() {
        unsafe {
            let slice = std::slice::from_raw_parts_mut(ptr, len as usize);
            drop(Box::from_raw(slice as *mut [u8]));
        }
    }
}
