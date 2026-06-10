use std::path::Path;
use std::sync::Arc;
use anyhow::{Result, Context};
use engine_lmdb::{LmdbEnv, read_neuron, iter_all_neurons};
use cluaizd_types::{NeuronId, UniversalNeuron};

/// Securely bridges the CLI with the core LMDB storage engine via FFI.
pub struct FfiBridge {
    env: Arc<LmdbEnv>,
}

impl FfiBridge {
    /// Opens a direct memory-mapped connection to the database shard.
    pub fn connect(shard_path: &Path) -> Result<Self> {
        let env = LmdbEnv::open(shard_path, 1024 * 1024 * 1024)
            .context("Failed to establish FFI link with Cluaizd Core Engine")?;
        Ok(Self {
            env: Arc::new(env),
        })
    }

    /// Fetches a specific neuron directly from LMDB.
    pub fn get_neuron(&self, id: &NeuronId) -> Result<UniversalNeuron> {
        read_neuron(&self.env, *id, None)
            .map_err(|e| anyhow::anyhow!("Storage error: {}", e))
    }

    /// Iterates and counts total neurons in the local shard.
    pub fn get_shard_stats(&self) -> Result<usize> {
        let neurons = iter_all_neurons(&self.env)
            .map_err(|e| anyhow::anyhow!("Storage error: {}", e))?;
        Ok(neurons.len())
    }

    /// Executes a CDQL query directly via FFI (no HTTP needed).
    pub fn run_cdql(&self, cdql: &str) -> Result<Vec<UniversalNeuron>> {
        // Parse the CDQL string into ops via genome crate
        let query = genome::cdql::parse(cdql)
            .map_err(|e| anyhow::anyhow!("CDQL Parse Error: {}", e))?;

        let mut results: Vec<UniversalNeuron> = Vec::new();

        // Walk the ops and execute against LMDB directly
        for op in &query.ops {
            match op {
                genome::cdql::parser::CdqlOp::Find { label, .. } => {
                    let all = iter_all_neurons(&self.env)
                        .map_err(|e| anyhow::anyhow!("Storage error: {}", e))?;
                    for n in all {
                        // Filter by payload_type label unless wildcard "*"
                        if label == "*" {
                            results.push(n);
                        } else {
                            let neuron_label = format!("{:?}", n.payload_type).to_lowercase();
                            if neuron_label == label.to_lowercase() {
                                results.push(n);
                            }
                        }
                    }
                }
                genome::cdql::parser::CdqlOp::FindById { id } => {
                    if let Ok(nid) = id.parse::<NeuronId>() {
                        if let Ok(n) = read_neuron(&self.env, nid, None) {
                            results.push(n);
                        }
                    }
                }
                genome::cdql::parser::CdqlOp::Limit(n) => {
                    results.truncate(*n);
                }
                _ => {
                    // Vector search, graph, geo, etc. require the full server
                    return Err(anyhow::anyhow!(
                        "This query step requires the full server (vector indexes, graph engine). \
                         Start the server first: `cluaizd-cli server start`"
                    ));
                }
            }
        }

        Ok(results)
    }
}

