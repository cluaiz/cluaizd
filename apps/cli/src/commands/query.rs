use anyhow::Result;
use std::path::Path;
use crate::engine::ffi_bridge::FfiBridge;
use crate::utils::printer::Printer;

/// `cluaizd-cli query "<CDQL>"` — Runs a CDQL query directly via FFI without needing the HTTP server.
pub fn run(shard_path: &Path, cdql: &str) -> Result<()> {
    let bridge = FfiBridge::connect(shard_path)?;

    println!("Executing CDQL via FFI: {}", cdql);

    let results = bridge.run_cdql(cdql)?;
    
    if results.is_empty() {
        println!("Query returned 0 results.");
    } else {
        println!("=== Query Results ({} neurons) ===", results.len());
        for neuron in &results {
            let json = serde_json::to_string_pretty(neuron)
                .unwrap_or_else(|_| "{}".to_string());
            println!("{}", json);
            println!("---");
        }
    }
    
    Ok(())
}
