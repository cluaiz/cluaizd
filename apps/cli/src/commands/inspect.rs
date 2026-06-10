use anyhow::{bail, Result};
use std::path::Path;
use cluaizd_types::NeuronId;
use crate::engine::ffi_bridge::FfiBridge;
use crate::utils::printer::Printer;

/// `cluaizd-cli db inspect <id>` — Fetches and prints a neuron by UUID.
pub fn run(shard_path: &Path, id_str: &str) -> Result<()> {
    let bridge = FfiBridge::connect(shard_path)?;

    let neuron_id = id_str
        .parse::<NeuronId>()
        .map_err(|e| anyhow::anyhow!("Invalid UUID '{}': {}", id_str, e))?;

    match bridge.get_neuron(&neuron_id) {
        Ok(neuron) => {
            Printer::print_json(&format!("Neuron {}", id_str), &neuron)?;
            Ok(())
        }
        Err(e) => {
            bail!("Error reading neuron '{}': {}", id_str, e);
        }
    }
}
