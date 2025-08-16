use axum::{
    routing::{get, post},
    Router,
};

use crate::env::state::AppState;

pub fn app() -> Router<AppState> {
    Router::new()
        .route("/", get(super::index::get))
        .route("/config", get(super::config::index::get))
        .route("/config/port", post(super::config::port::post))
}
