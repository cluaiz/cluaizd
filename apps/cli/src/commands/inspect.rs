use std::path::Path;
use anyhow::Result;
use uuid::Uuid;
use cluaizd_types::NeuronId;
use crate::engine::ffi_bridge::FfiBridge;
use crate::utils::printer::Printer;

pub fn run(shard_path: &Path, id_str: &str) -> Result<()> {
    let bridge = FfiBridge::connect(shard_path)?;
    
    // Note: Depends on how NeuronId is constructed. Using NeuronId::from_bytes for safety or from_str if implemented
    // The previous error showed `id.parse::<NeuronId>()` worked if FromStr was implemented.
    let neuron_id = id_str.parse::<NeuronId>().expect("Invalid UUID format");

    match bridge.get_neuron(&neuron_id) {
        Ok(neuron) => {
            Printer::print_json(&format!("Neuron {}", id_str), &neuron)?;
            Ok(())
        }
        Err(e) => {
            Printer::print_error(&format!("Error reading neuron: {}", e));
            Err(e)
        }
    }
}
