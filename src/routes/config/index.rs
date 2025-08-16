use axum::{extract::State, response::IntoResponse, Json};

use crate::env::state::AppState;

pub async fn get(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.read().await;
    Json(config.clone())
}
