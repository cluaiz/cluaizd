use std::sync::Arc;
use axum::{Json, body::Bytes, http::{StatusCode, HeaderMap}, response::IntoResponse, extract::State};
use serde::Serialize;
use tracing::warn;

use crate::routes::AppState;

/// Response body for `POST /ingest/stream`.
#[derive(Debug, Serialize)]
pub struct StreamIngestResponse {
    pub assigned_neuron_id: String,
    pub shard: &'static str,
    pub ttl_ms: u64,
}

/// `POST /ingest/stream` — Ingest a raw voltage/sensor byte stream.
///
/// This endpoint routes directly to the isolated `sensory_tissue.mdb` shard.
/// It bypasses the main LMDB database entirely.
///
/// ## Request Body
/// Raw bytes of the voltage or sensor stream. No JSON wrapping.
/// Content-Type: `application/octet-stream`
///
/// ## Headers
/// - `X-Device-Id` (optional): Source device identifier for logging.
pub async fn handle_stream(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    if body.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "stream body cannot be empty" })),
        ).into_response();
    }

    // Enforce Pacemaker pulse limit if configured
    let limit = state.write_rate_limit.load(std::sync::atomic::Ordering::SeqCst);
    if limit > 0 && body.len() as u32 > limit {
        warn!(size = body.len(), limit = limit, "BCI Sensory ingestion blocked by Pacemaker limit");
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(serde_json::json!({ "error": format!("Blocked by Pacemaker rate limit (max {} bytes)", limit) })),
        ).into_response();
    }


    let device_id = headers.get("x-device-id")
        .and_then(|h| h.to_str().ok());

    let dna = state.genome_registry.get_dna("sensory_stream");
    match state.sensory_shard.ingest_voltage_stream(body, device_id, dna) {
        Ok(neuron_id) => {
            (
                StatusCode::ACCEPTED,
                Json(StreamIngestResponse {
                    assigned_neuron_id: neuron_id.to_string(),
                    shard: "sensory_tissue",
                    ttl_ms: 30000,
                }),
            ).into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Sensory ingestion failed: {}", e) })),
            ).into_response()
        }
    }
}

