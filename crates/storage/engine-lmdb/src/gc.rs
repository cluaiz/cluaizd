use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::info;
use cluaizd_types::{StorageTier, UniversalNeuron};
use bytes::Bytes;
use zstd::stream::encode_all;
use rhai::{Engine, Scope, Map};

use crate::LmdbEnv;

/// Spawns the low-priority biological GC thread.
pub fn spawn_biological_gc(env: Arc<LmdbEnv>) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(3600)); // Run every hour
        loop {
            interval.tick().await;
            info!("Biological GC running 3-Tier storage sweeps...");

            let env_clone = env.clone();
            let _ = tokio::task::spawn_blocking(move || {
                run_gc_sweep(&env_clone)
            }).await;
        }
    });
}

/// Run a single GC sweep cycle to degrade expired or aged neurons dynamically based on node rules.
pub fn run_gc_sweep(env: &LmdbEnv) -> anyhow::Result<()> {
    let now_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    // Collect targets first to avoid write lock contention during iteration.
    let mut targets_to_update = Vec::new(); // Vec<(UniversalNeuron, i64)>
    let mut targets_to_delete = Vec::new();

    {
        let rtxn = env.env.read_txn()?;
        let iter = env.db.iter(&rtxn)?;
        for result in iter {
            let (key, mut neuron) = result?;
            
            let mut changed = false;
            let mut custom_compress_lvl = 3; // Default fallback

            // Evaluate Neuron-level DNA sequence
            if let Some(ref dna) = neuron.dna {
                if dna.engine == "rhai" {
                    let engine = Engine::new();
                    let mut scope = Scope::new();
                    
                    let age_ns = now_ns.saturating_sub(neuron.created_at_ns);
                    scope.push("age_ns", age_ns as i64);
                    scope.push("current_tier", format!("{:?}", neuron.tier));

                    if let Ok(dynamic_config) = rhai::serde::to_dynamic(&dna.parameters) {
                        scope.push_dynamic("config", dynamic_config);
                    }
                    
                    if let Some(lifecycle_script) = &dna.on_lifecycle {
                        if let Ok(result_map) = engine.eval_with_scope::<Map>(&mut scope, lifecycle_script) {
                            
                            // Explicit Deletion
                            if let Some(del) = result_map.get("delete_neuron").and_then(|v| v.as_bool().ok()) {
                                if del {
                                    targets_to_delete.push(neuron.id);
                                    continue;
                                }
                            }

                            // Explicit Tier Transitions
                            if let Some(new_tier) = result_map.get("new_tier").and_then(|v| v.clone().into_string().ok()) {
                                if new_tier == "Warm" && neuron.tier != StorageTier::Warm {
                                    neuron.tier = StorageTier::Warm;
                                    changed = true;
                                } else if new_tier == "Cold" && neuron.tier != StorageTier::Cold {
                                    neuron.tier = StorageTier::Cold;
                                    changed = true;
                                }
                            }

                            // Explicit Payload Clearing
                            if let Some(clear) = result_map.get("clear_payload").and_then(|v| v.as_bool().ok()) {
                                if clear && !neuron.raw_payload.is_empty() {
                                    neuron.raw_payload = Bytes::new();
                                    changed = true;
                                }
                            }

                            // Custom Compression Level
                            if let Some(lvl) = result_map.get("compress_level").and_then(|v| v.as_int().ok()) {
                                custom_compress_lvl = lvl;
                                changed = true;
                            }

                            // Explicit Edge Decay
                            if let Some(decay_factor) = result_map.get("edge_decay_factor").and_then(|v| v.as_float().ok()) {
                                if !neuron.adjacency.is_empty() {
                                    let prune_threshold = result_map.get("edge_prune_threshold").and_then(|v| v.as_float().ok()).unwrap_or(0.0) as f32;
                                    let original_len = neuron.adjacency.len();
                                    for edge in &mut neuron.adjacency {
                                        edge.weight *= decay_factor as f32;
                                    }
                                    neuron.adjacency.retain(|edge| edge.weight >= prune_threshold);
                                    if neuron.adjacency.len() != original_len {
                                        changed = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if changed {
                targets_to_update.push((neuron, custom_compress_lvl));
            }
        }
    }

    if !targets_to_update.is_empty() {
        let mut wtxn = env.env.write_txn()?;
        
        for (neuron, _compress_lvl) in targets_to_update {
            env.db.put(&mut wtxn, &neuron.id, &neuron)?;
        }

        for id in targets_to_delete {
            env.db.delete(&mut wtxn, &id)?;
        }

        wtxn.commit()?;
    }

    Ok(())
}

