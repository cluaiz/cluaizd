use std::sync::Arc;
use axum::{Json, extract::State, http::{StatusCode, HeaderMap}, response::IntoResponse};
use serde::{Deserialize, Serialize};

use cluaizd_types::{UniversalNeuron, StorageTier};
use crate::routes::AppState;
use genome::cdql::{parser::{parse, CdqlValue, CompareOp}, planner::{build_plan, PlanStep}};

/// Request payload for `POST /query`.
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    /// Target database/tenant (e.g. "default_sandbox")
    pub tenant_id: Option<String>,
    /// CDQL Universal Query String (e.g. `find *(name: "Aryan") -> get friends`)
    #[serde(alias = "CDQL")]
    pub cdql: Option<String>,
    /// Raw DNA query string (JSON match, for WASM DNA engine)
    pub query_string: Option<String>,
    /// Legacy: Search Vector for Vector/Geospatial searching
    pub query_vector: Option<Vec<f32>>,
    /// Top-K results to return (overridden by CDQL `limit` clause if present)
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub neuron: UniversalNeuron,
    pub score: f32,
    /// Which query path matched: "CDQL", "wasm", "rhai", "vector"
    pub matched_by: String,
}

/// `POST /query` — Universal Query Handler
///
/// Supports three query modes (in priority order):
/// 1. **CDQL** — `find *(name: "Aryan") -> get friends -> limit 10`
/// 2. **WASM DNA** — Raw query string passed into the Neuron's `.wasm` module
/// 3. **Rhai DNA** — Legacy script-based query via `on_index`
pub async fn handle_query(
    State(state): State<Arc<AppState>>,
    _headers: HeaderMap,
    Json(payload): Json<QueryRequest>,
) -> impl IntoResponse {
    let tenant_id = payload.tenant_id.as_deref().unwrap_or("default_sandbox");
    let global_limit = payload.limit.unwrap_or(100);

    let shard = match state.shard_manager.get_or_open_shard(tenant_id).await {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to open tenant shard: {}", e) })),
            ).into_response();
        }
    };

    let all_neurons = match engine_lmdb::iter_all_neurons(&shard.env) {
        Ok(n) => n,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() }))
        ).into_response(),
    };

    // ──────────────────────────────────────────────────────────────
    // PATH 1: CDQL Universal Query
    // ──────────────────────────────────────────────────────────────
    if let Some(ref cdql_str) = payload.cdql {
        return execute_cdql(cdql_str, all_neurons, global_limit).into_response();
    }

    // ──────────────────────────────────────────────────────────────
    // PATH 2 & 3: Legacy DNA-based queries (WASM or Rhai)
    // ──────────────────────────────────────────────────────────────
    let mut results: Vec<QueryResult> = Vec::new();
    let rhai_engine = genome::create_rhai_engine();

    for neuron in all_neurons {
        let mut score = 0.0f32;
        let mut matched_by = String::new();

        if let Some(ref dna) = neuron.dna {
            if let Some(index_script) = &dna.on_index {
                if dna.engine == "rhai" {
                    let mut scope = rhai::Scope::new();
                    if let Ok(dynamic_config) = rhai::serde::to_dynamic(&dna.parameters) {
                        scope.push_dynamic("config", dynamic_config);
                    }

                    let mut vec_arr = rhai::Array::new();
                    for v in &neuron.vector_data {
                        vec_arr.push(rhai::Dynamic::from(*v as rhai::FLOAT));
                    }
                    scope.push("vector_data", vec_arr);

                    if let Some(ref qv) = payload.query_vector {
                        let mut qv_arr = rhai::Array::new();
                        for v in qv {
                            qv_arr.push(rhai::Dynamic::from(*v as rhai::FLOAT));
                        }
                        scope.push("query_vector", qv_arr);
                    }

                    if let Ok(result_map) = rhai_engine.eval_with_scope::<rhai::Map>(&mut scope, index_script) {
                        if let Some(dist) = result_map.get("distance").and_then(|v| v.as_float().ok()) {
                            score = 1.0 / (1.0 + dist as f32);
                            matched_by = "rhai".to_string();
                        }
                        if let Some(s) = result_map.get("score").and_then(|v| v.as_float().ok()) {
                            score = s as f32;
                            matched_by = "rhai".to_string();
                        }
                    }

                } else if dna.engine == "wasm" {
                    if let Some(ref wasm_bytes) = dna.wasm_module {
                        let executor = genome::WasmExecutor::new();
                        let query_str = payload.query_string.as_deref().unwrap_or("{}");

                        if let Ok(res) = executor.execute_query(wasm_bytes, query_str, &neuron.raw_payload) {
                            if res > 0 {
                                score = 1.0;
                                matched_by = "wasm".to_string();
                            }
                        }
                    }
                }
            }
        }

        if score > 0.0 {
            results.push(QueryResult { neuron, score, matched_by });
        }
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(global_limit);

    (StatusCode::OK, Json(results)).into_response()
}

// ──────────────────────────────────────────────────────────────────
// CDQL Execution Engine
// ──────────────────────────────────────────────────────────────────

fn execute_cdql(
    cdql_str: &str,
    all_neurons: Vec<UniversalNeuron>,
    global_limit: usize,
) -> impl IntoResponse {
    // Step 1: Parse
    let query = match parse(cdql_str) {
        Ok(q) => q,
        Err(e) => return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("CDQL parse error: {}", e) })),
        ).into_response(),
    };

    // Step 2: Build plan
    let plan = match build_plan(&query) {
        Ok(p) => p,
        Err(e) => return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("CDQL plan error: {}", e) })),
        ).into_response(),
    };

    // Step 3: Execute plan
    let mut working_set: Vec<&UniversalNeuron> = all_neurons.iter().collect();
    let effective_limit = plan.limit.min(global_limit);

    for step in &plan.steps {
        match step {
            PlanStep::FastPathIdLookup { id } => {
                // Direct LMDB fetch bypasses all filters
                working_set = working_set.into_iter().filter(|n| n.id.to_string() == *id).collect();
            }

            PlanStep::ScanAll { label_filter, filters } => {
                working_set = working_set.into_iter().filter(|n| {
                    let payload_str = String::from_utf8_lossy(&n.raw_payload).to_string();
                    if let Some(ref label) = label_filter {
                        if label != "*" && !payload_str.to_lowercase().contains(&label.to_lowercase()) {
                            return false;
                        }
                    }
                    for filter in filters {
                        if !matches_filter(n, filter, &payload_str) { return false; }
                    }
                    true
                }).collect();
            }

            PlanStep::FilterResults { field, op, value } => {
                working_set = working_set.into_iter().filter(|n| {
                    let payload_str = String::from_utf8_lossy(&n.raw_payload).to_string();
                    matches_filter(n, &cluaizd_types_filter_ref(field, op, value), &payload_str)
                }).collect();
            }

            PlanStep::GraphTraverse { edge, min_hops: _, max_hops: _, min_weight: _ } => {
                // Mock implementation for deep traversal
                working_set = working_set.into_iter().filter(|n| {
                    !n.adjacency.is_empty() && !edge.is_empty()
                }).collect();
            }

            PlanStep::VectorScan { vector, metric: _ } => {
                working_set = working_set.into_iter().filter(|n| {
                    let dot: f32 = n.vector_data.iter().zip(vector.iter()).map(|(a, b)| a * b).sum();
                    dot > 0.0
                }).collect();
            }

            PlanStep::Limit(n) => {
                working_set.truncate(*n);
            }

            PlanStep::SortBy { field, ascending } => {
                working_set.sort_by(|a, b| {
                    let va = extract_json_field_str(&a.raw_payload, field);
                    let vb = extract_json_field_str(&b.raw_payload, field);
                    let cmp = va.cmp(&vb);
                    if *ascending { cmp } else { cmp.reverse() }
                });
            }

            // Stubs for new advanced features (to be fully implemented in Base DNAs later)
            PlanStep::Unwind { .. } => {}
            PlanStep::Project { .. } => {}
            PlanStep::ShortestPath { .. } => {}
            PlanStep::RelationalJoin { .. } => {}
            PlanStep::GroupBy { .. } => {}
            PlanStep::Aggregate { .. } => {}
            PlanStep::TimeWindow { .. } => {}
            PlanStep::FullTextSearch { .. } => {}
            PlanStep::GeoNear { .. } => {}
            PlanStep::RangeScan { .. } => {}
            PlanStep::ByteStream { .. } => {}
            PlanStep::InsertData { .. } => {} // Insertions are handled in the FFI or /neuron route
        }
    }

    working_set.truncate(effective_limit);

    let results: Vec<QueryResult> = working_set.into_iter().map(|n| {
        QueryResult {
            neuron: n.clone(),
            score: if n.tier == StorageTier::Hot { 1.0 } else { 0.5 },
            matched_by: "CDQL".to_string(),
        }
    }).collect();

    (StatusCode::OK, Json(results)).into_response()
}

/// Check if a neuron matches a single filter condition
fn matches_filter(
    _neuron: &UniversalNeuron,
    filter: &genome::cdql::parser::Filter,
    payload_str: &str,
) -> bool {
    // Try to parse payload as JSON for field extraction
    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(payload_str) {
        let field_val = json_val.get(&filter.field);

        match (&filter.op, &filter.value, field_val) {
            (CompareOp::Eq, CdqlValue::Text(t), Some(v)) => {
                v.as_str().map(|s| s == t.as_str()).unwrap_or(false)
            }
            (CompareOp::Eq, CdqlValue::Number(n), Some(v)) => {
                v.as_f64().map(|x| (x - n).abs() < 1e-9).unwrap_or(false)
            }
            (CompareOp::NotEq, CdqlValue::Text(t), Some(v)) => {
                v.as_str().map(|s| s != t.as_str()).unwrap_or(true)
            }
            (CompareOp::Gt, CdqlValue::Number(n), Some(v)) => {
                v.as_f64().map(|x| x > *n).unwrap_or(false)
            }
            (CompareOp::Lt, CdqlValue::Number(n), Some(v)) => {
                v.as_f64().map(|x| x < *n).unwrap_or(false)
            }
            (CompareOp::Gte, CdqlValue::Number(n), Some(v)) => {
                v.as_f64().map(|x| x >= *n).unwrap_or(false)
            }
            (CompareOp::Lte, CdqlValue::Number(n), Some(v)) => {
                v.as_f64().map(|x| x <= *n).unwrap_or(false)
            }
            (CompareOp::Contains, CdqlValue::Text(t), Some(v)) => {
                v.as_str().map(|s| s.contains(t.as_str())).unwrap_or(false)
            }
            // Field not found — no match
            (_, _, None) => false,
            _ => false,
        }
    } else {
        // Non-JSON payload: fallback to string contains match
        matches!(&filter.op, CompareOp::Contains | CompareOp::Eq) &&
            matches!(&filter.value, CdqlValue::Text(t) if payload_str.contains(t.as_str()))
    }
}

/// Helper to build a temporary Filter reference for inline use
fn cluaizd_types_filter_ref<'a>(
    field: &'a str,
    op: &'a CompareOp,
    value: &'a CdqlValue,
) -> genome::cdql::parser::Filter {
    genome::cdql::parser::Filter {
        field: field.to_string(),
        op: op.clone(),
        value: value.clone(),
    }
}

/// Extract a JSON field from raw_payload bytes as a string (for sorting)
fn extract_json_field_str(raw: &bytes::Bytes, field: &str) -> String {
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(raw) {
        json.get(field)
            .map(|v| v.to_string())
            .unwrap_or_default()
    } else {
        String::new()
    }
}
