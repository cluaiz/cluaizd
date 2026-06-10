use std::path::Path;
use anyhow::Result;
use crate::engine::ffi_bridge::FfiBridge;
use crate::utils::printer::Printer;

pub fn run(shard_path: &Path) -> Result<()> {
    match FfiBridge::connect(shard_path) {
        Ok(bridge) => {
            let count = bridge.get_shard_stats().unwrap_or(0);
            Printer::print_success("FFI Link Established.");
            println!("Shard Path: {:?}", shard_path);
            println!("Total Neurons: {}", count);
            Ok(())
        }
        Err(e) => {
            Printer::print_error(&format!("Could not open Cluaizd Core Engine at {:?}.", shard_path));
            Printer::print_error(&format!("Reason: {}", e));
            Err(e)
        }
    }
}
