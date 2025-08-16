use axum::{extract::State, response::IntoResponse, Json};

use crate::env::state::AppState;

pub async fn get(State(state): State<AppState>) -> impl IntoResponse {
    let new_config = state.reload_config().await.unwrap();
    Json(new_config)
}
