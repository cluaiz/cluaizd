use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn};

use cluaizd_errors::StorageError;
use cluaizd_types::UniversalNeuron;
use engine_lmdb::LmdbEnv;
use wal::{WalWriter, WalOperation};

/// Represents an active, open shard with its associated LMDB environment and WAL writer.
pub struct ActiveShard {
    /// The memory-mapped LMDB environment wrapped in an Arc.
    pub env: Arc<LmdbEnv>,
    /// The active WAL writer protected by a tokio Mutex.
    pub wal_writer: Arc<Mutex<WalWriter>>,
    /// The RAM ring-buffer for extreme high-frequency writes before LMDB.
    pub transit_lounge: crate::routes::transit::TransitLounge,
}

/// Dynamic registry managing multi-tenant database environments.
pub struct ShardManager {
    base_path: PathBuf,
    shards: RwLock<HashMap<String, Arc<ActiveShard>>>,
    telemetry: Arc<RwLock<heart::Telemetry>>,
}

impl ShardManager {
    /// Create a new ShardManager under the specified directory path.
    pub fn new(base_path: &Path, telemetry: Arc<RwLock<heart::Telemetry>>) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
            shards: RwLock::new(HashMap::new()),
            telemetry,
        }
    }

    /// Retrieve an already open shard or lazily initialize a new one.
    ///
    /// Running recovery on the target shard is handled automatically before returning
    /// to guarantee zero data loss.
    ///
    /// # Errors
    /// Returns `StorageError` if environment opening or WAL recovery fails.
    pub async fn get_or_open_shard(&self, tenant_id: &str) -> Result<Arc<ActiveShard>, StorageError> {
        // Attempt to read from the cached shards map first.
        {
            let shards_guard = self.shards.read().await;
            if let Some(shard) = shards_guard.get(tenant_id) {
                return Ok(Arc::clone(shard));
            }
        }

        // Lock for writing to initialize the new shard.
        let mut shards_guard = self.shards.write().await;
        // Double-checked locking pattern to avoid duplicate initialization.
        if let Some(shard) = shards_guard.get(tenant_id) {
            return Ok(Arc::clone(shard));
        }

        let tenant_dir = self.base_path.join(tenant_id);
        let db_path = tenant_dir.join("db");
        let wal_path = tenant_dir.join("wal");

        // 1. Open LMDB Environment (Map size configured to 1 GB) and wrap in Arc
        let env = Arc::new(LmdbEnv::open(&db_path, 1024 * 1024 * 1024)?);

        // 2. Perform WAL recovery
        wal::recover_from_wal(&wal_path, &mut |entry| {
            match entry.operation {
                WalOperation::Write { payload } => {
                    let neuron: UniversalNeuron = serde_json::from_slice(&payload)
                        .map_err(|e| StorageError::DeserializationFailed(e.to_string()))?;
                    engine_lmdb::write_neuron(&env, &neuron)?;
                }
                WalOperation::Delete { .. } => {}
            }
            Ok(())
        })?;

        // 3. Open WalWriter for subsequent transactions
        let wal_writer = Arc::new(Mutex::new(WalWriter::open(&wal_path)?));

        // 4. Spawn the low-priority biological GC thread for this shard (The Dreamer Engine Tier Shifter)
        engine_lmdb::spawn_biological_gc(Arc::clone(&env), Arc::clone(&self.telemetry));

        // 5. Initialize the Volatile Synaptic Transit Lounge
        let transit_lounge = crate::routes::transit::TransitLounge::new(100_000, Arc::clone(&env), Arc::clone(&wal_writer));

        let active_shard = Arc::new(ActiveShard {
            env,
            wal_writer,
            transit_lounge,
        });

        shards_guard.insert(tenant_id.to_string(), Arc::clone(&active_shard));
        Ok(active_shard)
    }

    /// Return the count of currently cached active shards.
    pub async fn active_shards_count(&self) -> usize {
        self.shards.read().await.len()
    }

    /// Return a snapshot of all active shards for background tasks like the Dreamer.
    pub async fn get_all_shards(&self) -> Vec<Arc<ActiveShard>> {
        let shards_guard = self.shards.read().await;
        shards_guard.values().cloned().collect()
    }

    /// Forcefully run a biological GC sweep cycle on all open database shards.
    pub async fn run_gc_sweep_on_all_shards(&self) -> Result<(), StorageError> {
        let shards_guard = self.shards.read().await;
        let spo2 = {
            let tel = self.telemetry.read().await;
            tel.spo2
        };

        for (tenant_id, shard) in shards_guard.iter() {
            info!(tenant = %tenant_id, "GC Sweep dynamically triggered on shard");
            if let Err(e) = engine_lmdb::run_gc_sweep(&shard.env, spo2) {
                warn!(tenant = %tenant_id, error = %e, "GC Sweep failed on shard");
            }
        }
        Ok(())
    }
}


