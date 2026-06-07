use std::ffi::{c_char, c_void, CString};
use std::slice;

use cnsdb_types::NeuronId;
use crate::env::LmdbEnv;
use crate::reader::read_neuron;

// Error codes:
//  0 = Success
// -1 = Null pointer provided
// -2 = Invalid UUID/NeuronId
// -3 = Neuron not found or storage error
// -4 = Serialization to JSON string failed

/// Read a neuron by its 16-byte UUID from the LMDB environment.
///
/// Returns the serialized neuron JSON string as a null-terminated C-string.
///
/// # Safety
/// - `env_ptr` must be a valid pointer to an initialized `LmdbEnv`.
/// - `id_ptr` must point to a 16-byte array containing the raw bytes of a `NeuronId`.
/// - `out_json` must point to a mutable `*mut c_char` address where the resulting JSON string pointer will be written.
/// - The caller is responsible for freeing the returned JSON string by calling `cnsdb_ffi_free_string`.
#[no_mangle]
pub unsafe extern "C" fn cnsdb_ffi_read_neuron(
    env_ptr: *mut c_void,
    id_ptr: *const u8,
    out_json: *mut *mut c_char,
) -> i32 {
    if env_ptr.is_null() || id_ptr.is_null() || out_json.is_null() {
        return -1;
    }

    // SAFETY: Caller must guarantee id_ptr points to at least 16 bytes.
    let id_bytes = slice::from_raw_parts(id_ptr, 16);
    let mut id_array = [0u8; 16];
    id_array.copy_from_slice(id_bytes);
    let id = NeuronId::from_bytes(id_array);


    // SAFETY: env_ptr was cast from *const LmdbEnv.
    let env = &*(env_ptr as *const LmdbEnv);

    // Perform read (no query model hash validation in raw FFI, return entire neuron)
    match read_neuron(env, id, None) {
        Ok(neuron) => {
            match serde_json::to_string(&neuron) {
                Ok(json_str) => {
                    match CString::new(json_str) {
                        Ok(c_str) => {
                            *out_json = c_str.into_raw();
                            0
                        }
                        Err(_) => -4,
                    }
                }
                Err(_) => -4,
            }
        }
        Err(_) => -3,
    }
}

/// Free a JSON string allocated by `cnsdb_ffi_read_neuron`.
///
/// # Safety
/// - `str_ptr` must be a valid pointer returned by `cnsdb_ffi_read_neuron` or null.
#[no_mangle]
pub unsafe extern "C" fn cnsdb_ffi_free_string(str_ptr: *mut c_char) {
    if !str_ptr.is_null() {
        // SAFETY: CString::from_raw safely reclaims and drops the string allocation.
        let _ = CString::from_raw(str_ptr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;
    use bytes::Bytes;
    use cnsdb_types::PayloadType;
    use cnsdb_types::UniversalNeuron;
    use crate::writer::write_neuron;

    #[test]
    fn test_ffi_read_neuron_succeeds() {
        let tmp_dir = std::env::temp_dir().join("cnsdb_test_ffi");
        let env = LmdbEnv::open(&tmp_dir, 10 * 1024 * 1024).expect("env open failed");

        let neuron = UniversalNeuron::new(
            Bytes::from("hello ffi"),
            [0.5f32; 16],
            [0u8; 32],
            PayloadType::Text,
        );
        write_neuron(&env, &neuron).expect("write failed");

        let env_ptr = &env as *const LmdbEnv as *mut c_void;
        let id_ptr = neuron.id.as_bytes().as_ptr();
        let mut out_json: *mut c_char = ptr::null_mut();

        // SAFETY: We provide valid stack references
        let result = unsafe {
            cnsdb_ffi_read_neuron(env_ptr, id_ptr, &mut out_json)
        };

        assert_eq!(result, 0);
        assert!(!out_json.is_null());

        // SAFETY: The string was allocated successfully and must be freed.
        unsafe {
            let c_str = std::ffi::CStr::from_ptr(out_json);
            let json_str = c_str.to_str().expect("invalid utf-8");
            assert!(json_str.contains(&neuron.id.to_string()));
            cnsdb_ffi_free_string(out_json);
        }

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
