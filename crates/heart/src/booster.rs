use std::fs;
use tracing::{info, warn};

/// Stores the state of the WASM Booster engine.
#[derive(Debug)]
pub struct BoosterState {
    pub wasm_bytes: Option<Vec<u8>>,
    /// Active mode ID: 0=Eco, 1=Balanced, 2=Performance, 3=Ultra, 4=UltraMaxBoost, 5=Auto, 6=Custom
    pub active_mode: u32,
}

impl Default for BoosterState {
    fn default() -> Self {
        Self {
            wasm_bytes: None,
            active_mode: 1, // Balanced default
        }
    }
}

impl BoosterState {
    /// Loads the booster.wasm from disk.
    pub fn load_from_disk(data_dir: &std::path::Path) -> Self {
        let path = data_dir.join("system_booster.wasm");
        let wasm_bytes = if path.exists() {
            if let Ok(bytes) = fs::read(&path) {
                info!("Loaded System Booster WASM from disk");
                Some(bytes)
            } else {
                warn!("Failed to read system_booster.wasm");
                None
            }
        } else {
            None
        };
        
        let mut state = Self::default();
        state.wasm_bytes = wasm_bytes;
        state
    }

    /// Saves the new wasm binary to disk and updates the state.
    pub fn save_wasm_to_disk(&mut self, data_dir: &std::path::Path, bytes: Vec<u8>) -> Result<(), std::io::Error> {
        let path = data_dir.join("system_booster.wasm");
        fs::write(path, &bytes)?;
        self.wasm_bytes = Some(bytes);
        Ok(())
    }
}
