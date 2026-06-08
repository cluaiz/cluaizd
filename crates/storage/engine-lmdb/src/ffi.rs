use std::ffi::{c_char, c_void, CString};
use std::slice;

use cluaizd_types::NeuronId;
use crate::env::LmdbEnv;
use crate::reader::read_neuron;

// Error codes:
//  0 = Success
// -1 = Null pointer provided
// -2 = Invalid UUID/NeuronId
// -3 = Neuron not found or storage error

#[repr(C)]
pub struct CluaizdFfiNeuron {
    pub id: [u8; 16],
    pub vector: [f32; 16],
    pub state_hash: [u8; 32],
    pub payload_ptr: *const u8,
    pub payload_len: usize,
}

/// Read a neuron by its 16-byte UUID from the LMDB environment.
///
/// Returns the raw binary struct directly from memory.
///
/// # Safety
/// - `env_ptr` must be a valid pointer to an initialized `LmdbEnv`.
/// - `id_ptr` must point to a 16-byte array containing the raw bytes of a `NeuronId`.
/// - `out_neuron` must point to a mutable `CluaizdFfiNeuron` struct where the data will be written.
#[no_mangle]
pub unsafe extern "C" fn cluaizd_ffi_read_neuron(
    env_ptr: *mut c_void,
    id_ptr: *const u8,
    out_neuron: *mut CluaizdFfiNeuron,
) -> i32 {
    if env_ptr.is_null() || id_ptr.is_null() || out_neuron.is_null() {
        return -1;
    }

    let id_bytes = slice::from_raw_parts(id_ptr, 16);
    let mut id_array = [0u8; 16];
    id_array.copy_from_slice(id_bytes);
    let id = NeuronId::from_bytes(id_array);

    let env = &*(env_ptr as *const LmdbEnv);

    match read_neuron(env, id, None) {
        Ok(neuron) => {
            // Write directly to the C struct without ANY JSON serialization
            let out = &mut *out_neuron;
            out.id.copy_from_slice(neuron.id.as_bytes());
            out.vector.copy_from_slice(&neuron.vector_data);
            out.state_hash.copy_from_slice(&neuron.model_creator_hash);
            
            // Expose the raw payload bytes directly
            out.payload_ptr = neuron.raw_payload.as_ptr();
            out.payload_len = neuron.raw_payload.len();
            
            // NOTE: The caller must read the payload before the neuron is dropped/transaction ends.
            // Since `read_neuron` currently returns an owned `UniversalNeuron`, this memory is valid
            // only as long as we keep it around. To make it TRULY zero-copy, the FFI should ideally 
            // return a read transaction handle. For now, we leak it so the caller can read it,
            // and provide a free function.
            std::mem::forget(neuron); 
            0
        }
        Err(_) => -3,
    }
}

/// Free the neuron payload leaked by `cluaizd_ffi_read_neuron`
#[no_mangle]
pub unsafe extern "C" fn cluaizd_ffi_free_neuron_payload(payload_ptr: *mut u8, payload_len: usize, capacity: usize) {
    if !payload_ptr.is_null() {
        let _ = Vec::from_raw_parts(payload_ptr, payload_len, capacity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;
    use bytes::Bytes;
    use cluaizd_types::PayloadType;
    use cluaizd_types::UniversalNeuron;
    use crate::writer::write_neuron;

    #[test]
    fn test_ffi_read_neuron_succeeds() {
        let tmp_dir = std::env::temp_dir().join("cluaizd_test_ffi");
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
        let mut out_neuron = CluaizdFfiNeuron {
            id: [0; 16],
            vector: [0.0; 16],
            state_hash: [0; 32],
            payload_ptr: ptr::null(),
            payload_len: 0,
        };

        // SAFETY: We provide valid stack references
        let result = unsafe {
            cluaizd_ffi_read_neuron(env_ptr, id_ptr, &mut out_neuron)
        };

        assert_eq!(result, 0);
        assert!(!out_neuron.payload_ptr.is_null());
        assert_eq!(out_neuron.vector[0], 0.5f32);

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
