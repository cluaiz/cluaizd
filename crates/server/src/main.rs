use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

mod routes;
mod dreamer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging.
    // Set RUST_LOG=debug to see all log output.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Cluaizd CLUAIZD Server v0.0.1 — starting");

    let shards_path = Path::new("data/shards");
    let sensory_path = Path::new("data/sensory");
    let wal_path = Path::new("data/wal");

    // ─────────────────────────────────────────────────────────
    // STEP 1: WAL Boot Recovery (Crash Safety Guarantee)
    // Replay any uncommitted WAL entries from a previous crash
    // into LMDB before we accept any new requests.
    // ─────────────────────────────────────────────────────────
    info!("Running WAL crash recovery (replaying uncommitted entries)...");
    let recovery_env = engine_lmdb::LmdbEnv::open(shards_path, 128 * 1024 * 1024 * 1024);
    match recovery_env {
        Ok(env) => {
            let mut replayed_count = 0usize;
            let result = wal::recover_from_wal(wal_path, &mut |entry| {
                if let wal::WalOperation::Write { payload } = &entry.operation {
                    if let Ok(neuron) = serde_json::from_slice::<cluaizd_types::UniversalNeuron>(payload) {
                        // Check if this neuron is already in LMDB (idempotent replay)
                        if engine_lmdb::read_neuron(&env, neuron.id, None).is_err() {
                            engine_lmdb::write_neuron(&env, &neuron)?;
                            replayed_count += 1;
                        }
                    }
                }
                Ok(())
            });
            match result {
                Ok(r) => info!(
                    total = r.total_entries,
                    replayed = r.replayed,
                    new_writes = replayed_count,
                    corrupt_skipped = r.skipped_corrupt,
                    "WAL boot recovery complete ✓"
                ),
                Err(e) => warn!("WAL recovery encountered errors (non-fatal): {}", e),
            }
        }
        Err(e) => warn!("Could not open default shard for WAL recovery (clean boot?): {}", e),
    }

    info!("Starting Cluaizd-HEART Autonomic Telemetry Engine");
    let data_dir = Path::new("data");
    let heart = heart::Heart::new(data_dir);
    heart.start_heartbeat();

    info!("Initializing CLUAIZD Shard Manager at {:?}", shards_path);
    let shard_manager = routes::ShardManager::new(shards_path, Arc::clone(&heart.telemetry));

    info!("Opening CLUAIZD Sensory Shard at {:?}", sensory_path);
    // Sensory shard capacity: 10,000 entries (ring buffer limit)
    let sensory_shard = engine_lmdb::SensoryShard::open(sensory_path, 10000)
        .map_err(|e| anyhow::anyhow!("Failed to open sensory shard: {}", e))?;

    info!("Loading Dynamic Genomes from ./genomes");
    let genome_registry = genome::GenomeRegistry::new();
    if let Err(e) = genome_registry.load_from_directory("genomes") {
        tracing::warn!("Failed to load some genomes: {}", e);
    }

    // Heart is already started above

    // Build the shared app state
    let state = routes::AppState::new(shard_manager, sensory_shard, genome_registry, Arc::clone(&heart.telemetry), Arc::clone(&heart.booster_state));

    // Start the Subconscious Dreaming Engine background task
    dreamer::spawn_dreamer(state.clone());

    // Build the Axum router with all routes.
    let app = routes::build_router(state);

    // Bind to local address — default port 7331.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:7331").await?;
    info!(addr = "0.0.0.0:7331", "Server listening — ready to accept requests");

    axum::serve(listener, app).await?;

    Ok(())
}
