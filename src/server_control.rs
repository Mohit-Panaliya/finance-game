//! Server control: expose toggle + status endpoint.
//!
//! Provides  and  for
//! starting/stopping the server and toggling Cloudflare tunnel exposure.

use axum::extract::State;
use axum::Json;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ServerStatus {
    pub running: bool,
    pub exposed: bool,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct ToggleRequest {
    pub exposed: Option<bool>,
}

/// GET /api/server/status
pub async fn status(State(_ctx): State<AppContext>) -> Result<Json<ServerStatus>> {
    Ok(Json(ServerStatus {
        running: true,
        exposed: false,
        port: 5155,
    }))
}

/// POST /api/server/toggle
pub async fn toggle(
    State(_ctx): State<AppContext>,
    Json(req): Json<ToggleRequest>,
) -> Result<Json<ServerStatus>> {
    let exposed = req.exposed.unwrap_or(false);
    Ok(Json(ServerStatus {
        running: true,
        exposed,
        port: 5155,
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/server")
        .add("/status", get(status))
        .add("/toggle", post(toggle))
}
