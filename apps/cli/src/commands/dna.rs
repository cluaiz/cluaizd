use anyhow::{Context, Result};
use std::path::Path;
use crate::utils::printer::Printer;

const ACTIVE_DNAS_PATH: &str = "active_dnas";

/// `cluaizd-cli dna list` — Lists all currently active WASM DNAs.
pub fn list() -> Result<()> {
    let entries = std::fs::read_dir(ACTIVE_DNAS_PATH)
        .context("Could not open active_dnas directory. Is Cluaizd initialized?")?;

    println!("=== Active WASM DNAs ===");
    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("wasm") {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let metadata = entry.metadata().ok();
            let size_kb = metadata.map(|m| m.len() / 1024).unwrap_or(0);
            println!("  [ACTIVE] {} ({} KB)", name, size_kb);
            count += 1;
        }
    }
    if count == 0 {
        println!("  (no active DNAs found)");
    }
    Ok(())
}

/// `cluaizd-cli dna deploy <path.wasm>` — Deploys a pre-compiled WASM file into the hot-reload cache.
pub fn deploy(wasm_path: &Path) -> Result<()> {
    if !wasm_path.exists() {
        anyhow::bail!("WASM file not found at: {:?}", wasm_path);
    }
    if wasm_path.extension().and_then(|e| e.to_str()) != Some("wasm") {
        anyhow::bail!("File must be a .wasm binary. Compile your Rust code with `cargo build --target wasm32-unknown-unknown` first.");
    }

    std::fs::create_dir_all(ACTIVE_DNAS_PATH)
        .context("Could not create active_dnas directory")?;

    let file_name = wasm_path.file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid WASM file path"))?;
    let dest = Path::new(ACTIVE_DNAS_PATH).join(file_name);

    std::fs::copy(wasm_path, &dest)
        .with_context(|| format!("Failed to copy WASM to {:?}", dest))?;

    Printer::print_success(&format!(
        "DNA '{}' deployed to active_dnas/. Hot-reload watcher will pick it up automatically.",
        file_name.to_string_lossy()
    ));
    Ok(())
}

/// `cluaizd-cli dna remove <name>` — Removes a DNA from the hot-reload cache.
pub fn remove(name: &str) -> Result<()> {
    let target = Path::new(ACTIVE_DNAS_PATH).join(name);
    if !target.exists() {
        anyhow::bail!("DNA '{}' not found in active_dnas/", name);
    }
    std::fs::remove_file(&target)
        .with_context(|| format!("Failed to remove {:?}", target))?;

    Printer::print_success(&format!("DNA '{}' removed from active_dnas/.", name));
    Ok(())
}
