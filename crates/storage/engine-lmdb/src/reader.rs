use tracing::debug;

use cluaizd_errors::StorageError;
use cluaizd_types::{NeuronId, UniversalNeuron};

use crate::env::LmdbEnv;

/// Read a `UniversalNeuron` from the LMDB database by its `NeuronId`.
///
/// # Arguments
/// * `env` — The open LMDB environment.
/// * `id` — The unique identifier of the neuron to fetch.
/// * `query_model_hash` — Optional SHA-256 hash of the requesting model.
///   If provided and it does not match the stored `model_creator_hash`,
///   an error is returned immediately to prevent cross-model vector misuse.
///
/// # Errors
/// Returns `StorageError::NeuronNotFound` if the ID does not exist.
/// Returns `StorageError::DeserializationFailed` if the stored bytes are corrupt.
pub fn read_neuron(
    env: &LmdbEnv,
    id: NeuronId,
    query_model_hash: Option<[u8; 32]>,
) -> Result<UniversalNeuron, StorageError> {
    let rtxn = env.read_txn()?;

    let bytes = match env.db.get(&rtxn, &id.as_bytes().as_slice())
        .map_err(|e| StorageError::ReadTxnFailed(e.to_string()))? {
        Some(b) => b,
        None => return Err(StorageError::NeuronNotFound(id)),
    };

    // Check for ZSTD magic number (0xFD2FB528 in little endian)
    // ZSTD frame starts with 28 B5 2F FD
    let is_compressed = bytes.len() >= 4 && bytes[0..4] == [0x28, 0xB5, 0x2F, 0xFD];

    let mut neuron: UniversalNeuron = if is_compressed {
        let decompressed = zstd::stream::decode_all(std::io::Cursor::new(bytes))
            .map_err(|e| StorageError::DeserializationFailed(format!("Decompression failed: {}", e)))?;
        serde_json::from_slice(&decompressed)
            .map_err(|e| StorageError::DeserializationFailed(format!("Deserialization error after decompress: {}", e)))?
    } else {
        serde_json::from_slice(bytes)
            .map_err(|e| StorageError::DeserializationFailed(format!("Deserialization error: {}", e)))?
    };

    // Validate model hash if a query hash was provided.
    if let Some(query_hash) = query_model_hash {
        if neuron.model_creator_hash != query_hash {
            return Err(StorageError::WriteTxnFailed(format!(
                "Model hash mismatch: stored={:?}, query={:?}. Re-embed with current model.",
                neuron.model_creator_hash, query_hash
            )));
        }
    }

    // Execute Circulation (on_read) DNA Hook
    let mut changed = false;
    if let Some(ref dna) = neuron.dna {
        if let Some(read_script) = &dna.on_read {
            if dna.engine == "rhai" {
                let engine = rhai::Engine::new();
                let mut scope = rhai::Scope::new();
                if let Ok(dynamic_config) = rhai::serde::to_dynamic(&dna.parameters) {
                    scope.push_dynamic("config", dynamic_config);
                }
                
                // Expose payload string if text
                if neuron.payload_type == cluaizd_types::PayloadType::Text {
                    if let Ok(text) = String::from_utf8(neuron.raw_payload.to_vec()) {
                        scope.push("payload", text);
                    }
                }

                if let Ok(result_map) = engine.eval_with_scope::<rhai::Map>(&mut scope, read_script) {
                    // Check for Edge Strengthening
                    if let Some(strengthen) = result_map.get("strengthen_factor").and_then(|v| v.as_float().ok()) {
                        if !neuron.adjacency.is_empty() {
                            for edge in &mut neuron.adjacency {
                                edge.weight *= strengthen as f32;
                                if edge.weight > 1.0 { edge.weight = 1.0; } // Cap at 1.0
                            }
                            changed = true;
                        }
                    }
                }
            } else if dna.engine == "wasm" {
                if let Some(ref wasm_bytes) = dna.wasm_module {
                    let executor = genome::WasmExecutor::new();
                    // Call the exported WASM function "on_read".
                    // If it returns a strengthen factor (e.g. integer percentage), apply it.
                    if let Ok(strengthen_percent) = executor.execute(wasm_bytes, "on_read") {
                        if strengthen_percent > 100 && !neuron.adjacency.is_empty() {
                            let strengthen_factor = strengthen_percent as f32 / 100.0;
                            for edge in &mut neuron.adjacency {
                                edge.weight *= strengthen_factor;
                                if edge.weight > 1.0 { edge.weight = 1.0; } // Cap at 1.0
                            }
                            changed = true;
                        }
                    }
                }
            }
        }
    }

    // Drop read transaction to free locks before a potential write transaction
    drop(rtxn);

    if changed {
        // We write the updated neuron back (background logic or inline)
        let _ = crate::writer::write_neuron(env, &neuron);
        debug!(neuron_id = %id, "Neuron connections strengthened dynamically on read");
    }

    debug!(neuron_id = %id, "Neuron read from LMDB");
    Ok(neuron)
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use cluaizd_types::PayloadType;

    use super::*;
    use crate::{env::LmdbEnv, writer::write_neuron};

    #[test]
    fn test_write_then_read_round_trip() {
        let tmp_dir = std::env::temp_dir().join("cluaizd_test_read");
        let env = LmdbEnv::open(&tmp_dir, 10 * 1024 * 1024).expect("env open failed");

        let model_hash = [42u8; 32];
        let neuron = UniversalNeuron::new(
            Bytes::from("hello cluaizd"),
            [1.0f32; 16],
            model_hash,
            PayloadType::Text,
        );
        let saved_id = neuron.id;

        write_neuron(&env, &neuron).expect("write failed");

        // Read back with matching model hash — should succeed
        let fetched = read_neuron(&env, saved_id, Some(model_hash)).expect("read failed");
        assert_eq!(fetched.id, saved_id);
        assert_eq!(fetched.raw_payload, Bytes::from("hello cluaizd"));

        // Read back with wrong model hash — should fail
        let wrong_hash = [0u8; 32];
        let result = read_neuron(&env, saved_id, Some(wrong_hash));
        assert!(result.is_err(), "Expected error on hash mismatch");

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}

/// Iterate over all neurons in the LMDB database.
/// This reads the entire database and decodes each neuron.
pub fn iter_all_neurons(env: &crate::env::LmdbEnv) -> Result<Vec<UniversalNeuron>, StorageError> {
    let rtxn = env.read_txn()?;
    let mut neurons = Vec::new();

    let iter = env.db.iter(&rtxn).map_err(|e| StorageError::ReadTxnFailed(e.to_string()))?;
    for result in iter {
        let (_k, v) = result.map_err(|e| StorageError::ReadTxnFailed(e.to_string()))?;
        
        let is_compressed = v.len() >= 4 && v[0..4] == [0x28, 0xB5, 0x2F, 0xFD];
        let neuron: UniversalNeuron = if is_compressed {
            let decompressed = zstd::stream::decode_all(std::io::Cursor::new(v))
                .map_err(|e| StorageError::DeserializationFailed(format!("Decompress failed: {}", e)))?;
            serde_json::from_slice(&decompressed)
                .map_err(|e| StorageError::DeserializationFailed(format!("Deserialize failed: {}", e)))?
        } else {
            serde_json::from_slice(v)
                .map_err(|e| StorageError::DeserializationFailed(format!("Deserialize failed: {}", e)))?
        };
        neurons.push(neuron);
    }

    Ok(neurons)
}
