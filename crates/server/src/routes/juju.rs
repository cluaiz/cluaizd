use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;

use crate::routes::AppState;

/// Coordinate representation for a neural node in the 3D/2D JUJU space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialCoordinates {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub tier: String,
}

/// The Live Spatial Map tracking node positions and connections.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpatialMap {
    /// Maps Neuron ID to its spatial coordinates.
    pub nodes: HashMap<String, SpatialCoordinates>,
    /// Maps Source Neuron ID to a list of Target Neuron IDs.
    pub edges: HashMap<String, Vec<String>>,
}

/// `GET /juju/state` — Fetch the live spatial map for JUJU Canvas rendering.
pub async fn handle_juju_state(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let map = state.spatial_map.read().await;
    (StatusCode::OK, Json(map.clone())).into_response()
}
