use tracing::debug;

use cnsdb_errors::StorageError;
use cnsdb_types::UniversalNeuron;

use crate::env::LmdbEnv;

/// Write a `UniversalNeuron` to the LMDB database.
///
/// Serializes the neuron to JSON bytes and stores it under its `NeuronId` key.
/// In a future phase, this will use zero-copy binary serialization (rkyv/flatbuffers).
///
/// # Errors
/// Returns `StorageError` if the write transaction or serialization fails.
pub fn write_neuron(env: &LmdbEnv, neuron: &UniversalNeuron) -> Result<(), StorageError> {
    let key = neuron.id.as_bytes().to_vec();

    let value =
        serde_json::to_vec(neuron).map_err(|e| StorageError::SerializationFailed(e.to_string()))?;

    let mut wtxn = env.write_txn()?;

    env.db
        .put(&mut wtxn, &key, &value)
        .map_err(|e| StorageError::WriteTxnFailed(e.to_string()))?;

    wtxn.commit().map_err(|e| StorageError::WriteTxnFailed(e.to_string()))?;

    debug!(neuron_id = %neuron.id, "Neuron written to LMDB");
    Ok(())
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use cnsdb_types::PayloadType;

    use super::*;
    use crate::env::LmdbEnv;

    #[test]
    fn test_write_neuron_succeeds() {
        let tmp_dir = std::env::temp_dir().join("cnsdb_test_write");
        let env = LmdbEnv::open(&tmp_dir, 10 * 1024 * 1024).expect("env open failed");

        let neuron = UniversalNeuron::new(
            Bytes::from("test payload"),
            [0.1f32; 16],
            [0u8; 32],
            PayloadType::Text,
        );

        write_neuron(&env, &neuron).expect("write failed");
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
