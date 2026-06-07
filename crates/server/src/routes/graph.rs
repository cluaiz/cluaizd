use std::sync::Arc;
use std::collections::{VecDeque, HashSet};
use axum::{Json, extract::{Path, State, Query}, http::{StatusCode, HeaderMap}, response::IntoResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use cnsdb_types::{NeuronId, UniversalNeuron};
use crate::routes::AppState;

#[derive(Deserialize)]
pub struct TraverseParams {
    pub depth: Option<u32>,
}

#[derive(Serialize)]
pub struct Subgraph {
    pub nodes: Vec<UniversalNeuron>,
}

/// `GET /graph/{id}/traverse` — Fetch a node and its network (neighbors) up to Depth N.
pub async fn handle_traverse(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id_str): Path<String>,
    Query(params): Query<TraverseParams>,
) -> impl IntoResponse {
    let uuid = match Uuid::parse_str(&id_str) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid neuron ID format (expected UUID v7)" })),
            ).into_response();
        }
    };

    let root_id = NeuronId::from_bytes(*uuid.as_bytes());
    let max_depth = params.depth.unwrap_or(2);

    let tenant_id = match headers.get("x-tenant-id") {
        Some(val) => val.to_str().unwrap_or("default_sandbox"),
        None => "default_sandbox",
    };

    let shard = match state.shard_manager.get_or_open_shard(tenant_id).await {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to open tenant shard: {}", e) })),
            ).into_response();
        }
    };

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut subgraph_nodes = Vec::new();

    queue.push_back((root_id, 0));
    visited.insert(root_id);

    while let Some((curr_id, current_depth)) = queue.pop_front() {
        if current_depth > max_depth {
            continue;
        }

        let curr_neuron = match engine_lmdb::read_neuron(&shard.env, curr_id, None) {
            Ok(n) => n,
            Err(_) => continue,
        };

        // Explore neighbors if not at max depth
        if current_depth < max_depth {
            let mut follow_edges = Vec::new();
            
            // Check DNA hook on_traverse
            let mut used_dna_hook = false;
            if let Some(ref dna) = curr_neuron.dna {
                if let Some(traverse_script) = &dna.on_traverse {
                    if dna.engine == "rhai" {
                        let engine = rhai::Engine::new();
                        
                        for edge in &curr_neuron.adjacency {
                            let mut scope = rhai::Scope::new();
                            scope.push("edge_weight", edge.weight as f64);
                            if let Ok(dynamic_config) = rhai::serde::to_dynamic(&dna.parameters) {
                                scope.push_dynamic("config", dynamic_config);
                            }
                            
                            used_dna_hook = true;
                            let mut follow = true; // default
                            
                            if let Ok(result_map) = engine.eval_with_scope::<rhai::Map>(&mut scope, traverse_script) {
                                if let Some(f) = result_map.get("follow").and_then(|v| v.as_bool().ok()) {
                                    follow = f;
                                }
                            }
                            
                            if follow {
                                follow_edges.push(edge.target_id);
                            }
                        }
                    } else if dna.engine == "wasm" {
                        if let Some(ref wasm_bytes) = dna.wasm_module {
                            let executor = genome::WasmExecutor::new();
                            for edge in &curr_neuron.adjacency {
                                used_dna_hook = true;
                                let mut follow = true;
                                // we'd pass edge weight or id to wasm here
                                if let Ok(res) = executor.execute(wasm_bytes, "on_traverse") {
                                    follow = res > 0;
                                }
                                if follow {
                                    follow_edges.push(edge.target_id);
                                }
                            }
                        }
                    }
                }
            }

            if !used_dna_hook {
                for edge in &curr_neuron.adjacency {
                    follow_edges.push(edge.target_id);
                }
            }

            for target_id in follow_edges {
                if !visited.contains(&target_id) {
                    visited.insert(target_id);
                    queue.push_back((target_id, current_depth + 1));
                }
            }
        }
        
        subgraph_nodes.push(curr_neuron);
    }

    (StatusCode::OK, Json(Subgraph { nodes: subgraph_nodes })).into_response()
}
