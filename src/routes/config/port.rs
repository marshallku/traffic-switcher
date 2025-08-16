use axum::{extract::State, response::IntoResponse, Json};
use hyper::StatusCode;
use serde::Deserialize;

use crate::env::state::AppState;

#[derive(Deserialize)]
pub struct UpdatePortRequest {
    pub service: String,
    pub port: u16,
}

pub async fn post(
    State(state): State<AppState>,
    Json(req): Json<UpdatePortRequest>,
) -> impl IntoResponse {
    if req.port == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid port number"
            })),
        );
    }

    match state.update_service_port(&req.service, req.port).await {
        Ok(old_port) => {
            if let Err(e) = state.save_config().await {
                log::error!("Failed to save config: {}", e);
            }

            log::info!(
                "Updated {} from port {} to {}",
                req.service,
                old_port,
                req.port
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "message": format!("Service '{}' updated from port {} to {}", req.service, old_port, req.port),
                    "previous_port": old_port,
                    "current_port": req.port
                })),
            )
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": e
            })),
        ),
    }
}
