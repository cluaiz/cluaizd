use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use crate::routes::shard_manager::ShardManager;
use engine_lmdb::SensoryShard;
use genome::GenomeRegistry;
use heart::{Telemetry, booster::BoosterState};
use tokio::sync::RwLock;
use crate::routes::juju::SpatialMap;

/// Shared application state for all Axum routes.
pub struct AppState {
    /// Global registry for all loaded DNA genomes.
    pub genome_registry: GenomeRegistry,
    /// Registry managing physical sharded database environments.
    pub shard_manager: ShardManager,
    /// Shard for BCI/sensory high-frequency data streams.
    pub sensory_shard: SensoryShard,
    /// Dynamic write rate/pulse size limit (in bytes) configured via Pacemaker UI.
    pub write_rate_limit: AtomicU32,
    /// Real-time biological telemetry representing the system's hardware state.
    pub heart_telemetry: Arc<RwLock<Telemetry>>,
    /// Booster WASM State
    pub booster_state: Arc<RwLock<BoosterState>>,
    /// Live Spatial Coordinates Map for JUJU frontend rendering.
    pub spatial_map: Arc<RwLock<SpatialMap>>,
}

impl AppState {
    pub fn new(shard_manager: ShardManager, sensory_shard: SensoryShard, genome_registry: GenomeRegistry, heart_telemetry: Arc<RwLock<Telemetry>>, booster_state: Arc<RwLock<BoosterState>>) -> Arc<Self> {
        Arc::new(Self {
            genome_registry,
            shard_manager,
            sensory_shard,
            write_rate_limit: AtomicU32::new(0), // 0 means unrestricted/no limit
            heart_telemetry,
            booster_state,
            spatial_map: Arc::new(RwLock::new(SpatialMap::default())),
        })
    }
}


