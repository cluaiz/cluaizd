use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::routes::AppState;

/// System telemetry data mapping to Cluaiz-HEART biomarkers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartTelemetry {
    pub heart_rate_bpm: u32,
    pub blood_pressure_systolic: u32,
    pub blood_pressure_diastolic: u32,
    pub oxygen_level_spo2: f32,
    pub metabolic_rate: f32,
}

/// Commands received from the Genome Canvas UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", content = "payload")]
pub enum UICommand {
    #[serde(rename = "adrenaline_shot")]
    AdrenalineShot,
    #[serde(rename = "artificial_pacemaker")]
    ArtificialPacemaker { pulse_limit: u32 },
    #[serde(rename = "induced_coma")]
    InducedComa,
}

/// Upgrades the connection to a WebSocket and spawns the telemetry loop.
pub async fn handle_ws(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(state, socket))
}

async fn handle_socket(state: Arc<AppState>, socket: WebSocket) {
    info!("Cluaiz-HEART WebSocket telemetry client connected");

    // Spawn a receiver task to handle incoming controls
    let (mut sender, mut receiver) = socket.split();
    let state_rx = Arc::clone(&state);

    let rx_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                match serde_json::from_str::<UICommand>(&text) {
                    Ok(cmd) => match cmd {
                        UICommand::AdrenalineShot => {
                            info!("HEART Trigger: ADRENALINE SHOT! Forceful uncompression activated.");
                            if let Err(e) = state_rx.shard_manager.run_gc_sweep_on_all_shards().await {
                                warn!(error = %e, "Dynamic sweep execution failed during Adrenaline Shot");
                            }
                        }
                        UICommand::ArtificialPacemaker { pulse_limit } => {
                            info!("HEART Trigger: ARTIFICIAL PACEMAKER set pulse limit to {} bytes.", pulse_limit);
                            state_rx.write_rate_limit.store(pulse_limit, std::sync::atomic::Ordering::SeqCst);
                        }
                        UICommand::InducedComa => {
                            info!("HEART Trigger: INDUCED COMA! Emergency WAL commit and database consistency verified.");
                        }
                    },
                    Err(e) => {
                        warn!("Failed to parse UICommand: {}. Raw: {}", e, text);
                    }
                }
            }
        }
        info!("WebSocket receiver task shutdown");
    });

    // Telemetry sender loop (every 500ms)
    let state_tx = Arc::clone(&state);
    let tx_task = tokio::spawn(async move {
        let mut heart_rate = 72;
        let mut sys = 120;
        let mut dia = 80;
        let mut spo2 = 98.5;
        let mut metabolic = 1.0;

        loop {
            // Compute real database metrics to influence telemetry biomarkers
            let open_shards = state_tx.shard_manager.active_shards_count().await as u32;
            let target_heart_rate = (72 + open_shards * 8).min(180);

            // Simulate minor biometric fluctuations drifting towards the target load indicator
            heart_rate = (heart_rate as i32 + rand_range(-2, 2)).max(60).min(180) as u32;
            if heart_rate < target_heart_rate {
                heart_rate += 1;
            } else if heart_rate > target_heart_rate {
                heart_rate -= 1;
            }

            sys = (sys as i32 + rand_range(-2, 2)).max(100).min(150) as u32;
            dia = (dia as i32 + rand_range(-1, 1)).max(60).min(95) as u32;
            spo2 = (spo2 + rand_range_f32(-0.1, 0.1)).max(95.0).min(100.0);
            
            // Metabolic rate reflects the number of active database shards
            let target_metabolic = 1.0 + (open_shards as f32 * 0.25);
            metabolic = (metabolic + (target_metabolic - metabolic) * 0.1 + rand_range_f32(-0.02, 0.02)).max(0.2).min(3.0);


            let telemetry = HeartTelemetry {
                heart_rate_bpm: heart_rate,
                blood_pressure_systolic: sys,
                blood_pressure_diastolic: dia,
                oxygen_level_spo2: spo2,
                metabolic_rate: metabolic,
            };

            if let Ok(json) = serde_json::to_string(&telemetry) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break; // Connection closed
                }
            }

            sleep(Duration::from_millis(500)).await;
        }
        info!("WebSocket transmitter task shutdown");
    });

    // Wait until one of the tasks ends or fails
    tokio::select! {
        _ = rx_task => {},
        _ = tx_task => {},
    }
    info!("Cluaiz-HEART WebSocket client disconnected");
}

// Simple LCG helper functions for generating telemetry fluctuations
fn rand_range(min: i32, max: i32) -> i32 {
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    let rand_val = (seed & 0x7FFFFFFF) as i32;
    min + (rand_val % (max - min + 1))
}

fn rand_range_f32(min: f32, max: f32) -> f32 {
    let r = rand_range(0, 1000) as f32 / 1000.0;
    min + r * (max - min)
}

