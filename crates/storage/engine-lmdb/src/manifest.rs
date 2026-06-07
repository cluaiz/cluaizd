use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("Failed to parse manifest JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Trait '{name}' has invalid offset range: start={start}, end={end}, data_len={data_len}")]
    InvalidOffsetRange {
        name: String,
        start: usize,
        end: usize,
        data_len: usize,
    },
    #[error("Cryptographic hash mismatch for trait '{name}': expected {expected}, calculated {calculated}")]
    HashMismatch {
        name: String,
        expected: String,
        calculated: String,
    },
}

/// A specific genetic trait defined in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneTrait {
    pub name: String,
    pub description: String,
    /// Start byte offset in the binary DNA sequence
    pub start_offset: usize,
    /// End byte offset in the binary DNA sequence
    pub end_offset: usize,
    /// Expected SHA-256 hex string hash of the segment
    pub expected_hash: String,
}

/// The Gene Registry Manifest mapping binary DNA blocks to structured features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneRegistryManifest {
    pub version: String,
    pub traits: Vec<GeneTrait>,
}

impl GeneRegistryManifest {
    /// Create a new empty manifest with a specific version.
    pub fn new(version: String) -> Self {
        Self {
            version,
            traits: Vec::new(),
        }
    }

    /// Add a trait to the registry.
    pub fn add_trait(&mut self, gene_trait: GeneTrait) {
        self.traits.push(gene_trait);
    }

    /// Validates a raw binary DNA block against the manifest.
    ///
    /// For each trait, it extracts the sub-slice defined by `start_offset..end_offset`,
    /// calculates its SHA-256 hash, and compares it to the expected hash in the manifest.
    pub fn validate_dna(&self, dna_bytes: &[u8]) -> Result<(), ManifestError> {
        for t in &self.traits {
            if t.start_offset > t.end_offset || t.end_offset > dna_bytes.len() {
                return Err(ManifestError::InvalidOffsetRange {
                    name: t.name.clone(),
                    start: t.start_offset,
                    end: t.end_offset,
                    data_len: dna_bytes.len(),
                });
            }

            let segment = &dna_bytes[t.start_offset..t.end_offset];
            let mut hasher = Sha256::new();
            hasher.update(segment);
            let result = hasher.finalize();
            let calculated_hash = hex::encode(result);

            if calculated_hash != t.expected_hash {
                return Err(ManifestError::HashMismatch {
                    name: t.name.clone(),
                    expected: t.expected_hash.clone(),
                    calculated: calculated_hash,
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dna_validation() {
        let dna = b"hello_world_bhojpuri_dialect_movement_reflexes_12345";
        // Bhojuri dialect trait segment: "bhojpuri_dialect"
        // Start: 12, End: 28
        // Let's compute its SHA-256 hash:
        let mut hasher = Sha256::new();
        hasher.update(b"bhojpuri_dialect");
        let hash_hex = hex::encode(hasher.finalize());

        let mut manifest = GeneRegistryManifest::new("1.0.0".to_string());
        manifest.add_trait(GeneTrait {
            name: "Bhojpuri Dialect".to_string(),
            description: "Speech synthesis patterns for Bhojpuri language.".to_string(),
            start_offset: 12,
            end_offset: 28,
            expected_hash: hash_hex,
        });

        assert!(manifest.validate_dna(dna).is_ok());

        // Test hash mismatch
        let mut invalid_manifest = manifest.clone();
        invalid_manifest.traits[0].expected_hash = "wronghash".to_string();
        assert!(invalid_manifest.validate_dna(dna).is_err());
    }
}
