use axum::{
    Json,
    extract::{Path, State, Multipart},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use serde::Serialize;
use tracing::info;

use crate::routes::AppState;

#[derive(Serialize)]
pub struct BoosterResponse {
    pub status: String,
    pub message: String,
}

/// `POST /booster/upload` — Upload a new system_booster.wasm binary.
pub async fn handle_upload_booster(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut wasm_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if field.name() == Some("booster_wasm") {
            if let Ok(bytes) = field.bytes().await {
                wasm_bytes = Some(bytes.to_vec());
            }
        }
    }

    if let Some(bytes) = wasm_bytes {
        let mut booster_state = state.booster_state.write().await;
        let data_dir = std::path::Path::new("data");
        if let Err(e) = booster_state.save_wasm_to_disk(data_dir, bytes) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BoosterResponse {
                    status: "error".to_string(),
                    message: format!("Failed to save booster.wasm: {}", e),
                }),
            ).into_response();
        }

        info!("Successfully uploaded and replaced system_booster.wasm");
        (
            StatusCode::OK,
            Json(BoosterResponse {
                status: "success".to_string(),
                message: "Successfully loaded system_booster.wasm".to_string(),
            }),
        ).into_response()
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(BoosterResponse {
                status: "error".to_string(),
                message: "Missing 'booster_wasm' field in multipart form".to_string(),
            }),
        ).into_response()
    }
}

/// `POST /booster/mode/{mode}` — Change the active booster mode ID.
pub async fn handle_change_mode(
    State(state): State<Arc<AppState>>,
    Path(mode_str): Path<String>,
) -> impl IntoResponse {
    let mode_id = match mode_str.to_lowercase().as_str() {
        "eco" => 0,
        "balanced" => 1,
        "performance" => 2,
        "ultra" => 3,
        "ultramaxboost" => 4,
        "auto" => 5,
        "custom" => 6,
        _ => return (
            StatusCode::BAD_REQUEST,
            Json(BoosterResponse {
                status: "error".to_string(),
                message: format!("Invalid mode: {}", mode_str),
            }),
        ).into_response(),
    };

    let mut booster_state = state.booster_state.write().await;
    booster_state.active_mode = mode_id;
    
    info!(mode_id = mode_id, "Switched active booster mode");

    (
        StatusCode::OK,
        Json(BoosterResponse {
            status: "success".to_string(),
            message: format!("Switched booster mode to {}", mode_str),
        }),
    ).into_response()
}
