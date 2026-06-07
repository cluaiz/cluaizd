use std::sync::Arc;
use axum::{Json, http::{StatusCode, HeaderMap}, response::IntoResponse, extract::State};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tracing::warn;

use cluaizd_types::{PayloadType, UniversalNeuron, NeuronDna, NeuronEdge};
use crate::routes::AppState;

/// API request representation of a Neuron Edge.
#[derive(Debug, Deserialize)]
pub struct WriteNeuronEdgeRequest {
    pub target_id: String,
    pub weight: f32,
}

/// Request body for `POST /neuron`.
///
/// The AI model or client is responsible for computing the vector.
/// CLUAIZD only stores what it receives — it never generates vectors internally.
#[derive(Debug, Deserialize)]
pub struct WriteNeuronRequest {
    /// The raw payload as a UTF-8 string (for text) or base64 (for binary).
    pub raw_payload: String,
    /// The 16-dimensional hardware footprint vector from the external model.
    pub vector_data: [f32; 16],
    /// SHA-256 hash (hex string) of the model that generated `vector_data`.
    pub model_creator_hash: String,
    /// The type of data (text, audio, video, code, binary).
    pub payload_type: String,
    /// Optional dynamic ruleset configuration.
    pub dna: Option<NeuronDna>,
    /// Optional initial weighted edges to connect to.
    pub adjacency: Option<Vec<WriteNeuronEdgeRequest>>,
}

/// Response body for a successful neuron write.
#[derive(Debug, Serialize)]
pub struct WriteNeuronResponse {
    pub neuron_id: String,
    pub status: &'static str,
}

/// `POST /neuron` — Write a new neuron to the database.
pub async fn handle_write(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<WriteNeuronRequest>,
) -> impl IntoResponse {
    // Validate vector dimensions.
    if body.vector_data.iter().any(|v| !v.is_finite()) {
        warn!("Rejected write: vector contains NaN or Infinity");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "vector_data contains non-finite values" })),
        ).into_response();
    }

    // Parse model hash from hex.
    let model_hash_bytes = match hex::decode(&body.model_creator_hash) {
        Ok(b) if b.len() == 32 => {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&b);
            arr
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "model_creator_hash must be a 64-char hex string (SHA-256)" })),
            ).into_response();
        }
    };

    let payload_type = match body.payload_type.as_str() {
        "text"           => PayloadType::Text,
        "audio"          => PayloadType::Audio,
        "video"          => PayloadType::Video,
        "code"           => PayloadType::Code,
        "voltage_stream" => PayloadType::VoltageStream,
        _                => PayloadType::Binary,
    };

    let mut neuron = UniversalNeuron::new(
        Bytes::from(body.raw_payload.into_bytes()),
        body.vector_data,
        model_hash_bytes,
        payload_type,
    );

    // Hydrate dynamic ruleset
    neuron.dna = body.dna;

    // Hydrate adjacency edges
    if let Some(req_edges) = body.adjacency {
        let mut edges = Vec::new();
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        for e in req_edges {
            if let Ok(uuid) = uuid::Uuid::parse_str(&e.target_id) {
                edges.push(NeuronEdge {
                    target_id: cluaizd_types::NeuronId::from_bytes(*uuid.as_bytes()),
                    weight: e.weight,
                    last_accessed_ns: now_ns,
                });
            }
        }
        neuron.adjacency = edges;
    }

    let neuron_id = neuron.id.to_string();

    // Resolve tenant ID from headers
    let tenant_id = match headers.get("x-tenant-id") {
        Some(val) => val.to_str().unwrap_or("default_sandbox"),
        None => "default_sandbox",
    };

    // Open/retrieve sharded database
    let shard = match state.shard_manager.get_or_open_shard(tenant_id).await {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, tenant = %tenant_id, "Failed to resolve or open tenant shard");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to open tenant shard: {}", e) })),
            ).into_response();
        }
    };

    // Retrieve current telemetry for the DNA script
    let (bp, spo2) = {
        let tel = state.heart_telemetry.read().await;
        (tel.bp_systolic, tel.spo2)
    };

    // Execute DNA on_write hook
    if let Some(dna) = &neuron.dna {
        if let Some(write_script) = &dna.on_write {
            if dna.engine == "rhai" {
                let engine = rhai::Engine::new();
                let mut scope = rhai::Scope::new();
                
                let metrics = rhai::Map::from([
                    ("bp".into(), (bp as i64).into()),
                    ("spo2".into(), (spo2 as i64).into())
                ]);
                scope.push("system_metrics", metrics);
                
                if let Ok(result_map) = engine.eval_with_scope::<rhai::Map>(&mut scope, write_script) {
                    if let Some(action) = result_map.get("action").map(|v| v.to_string()) {
                        if action == "Defer" {
                            tracing::warn!(bp=bp, spo2=spo2, neuron_id=%neuron_id, "DNA deferred write processing (Lazy Execution)");
                            // In a real defer we might write to an append-only lazy queue instead of primary LMDB.
                            // For now, we simulate this by ONLY writing to WAL and skipping LMDB insert.
                            if let Err(e) = shard.wal_writer.lock().await.append_write(&neuron) {
                                return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": format!("WAL write failure: {}", e) }))).into_response();
                            }
                            return (
                                StatusCode::ACCEPTED,
                                Json(WriteNeuronResponse { neuron_id, status: "deferred" }),
                            ).into_response();
                        }
                    }
                }
            }
        }
    }

    // Append mutation to the WAL first
    if let Err(e) = shard.wal_writer.lock().await.append_write(&neuron) {
        warn!(error = %e, tenant = %tenant_id, "Failed to write to WAL");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("WAL write failure: {}", e) })),
        ).into_response();
    }

    // Write to LMDB database
    if let Err(e) = engine_lmdb::write_neuron(&shard.env, &neuron) {
        warn!(error = %e, tenant = %tenant_id, "Failed to write to LMDB");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("LMDB write failure: {}", e) })),
        ).into_response();
    }

    (
        StatusCode::CREATED,
        Json(WriteNeuronResponse {
            neuron_id,
            status: "created",
        }),
    ).into_response()
}


