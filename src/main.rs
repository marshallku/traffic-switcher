use std::net::SocketAddr;

use axum::Router;
use env::state::AppState;
use routes::app::app;
use tokio::signal;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{info, Level};
use utils::log::trace_layer_on_request;

use crate::routes::proxy::proxy_handler;

mod env;
mod routes;
mod utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let state = AppState::new().await;
    let api_addr = SocketAddr::from(([0, 0, 0, 0], state.port));
    let proxy_addr = SocketAddr::from(([0, 0, 0, 0], state.proxy_port));
    let api_app = app()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_request(trace_layer_on_request),
        )
        .with_state(state.clone());
    let proxy_app = Router::new().fallback(proxy_handler).with_state(state);

    info!("API server listening on http://{}", api_addr);
    info!("Proxy server listening on http://{}", proxy_addr);

    let api_server = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(api_addr)
            .await
            .expect("Failed to bind API server");
        axum::serve(listener, api_app)
            .with_graceful_shutdown(handle_shutdown())
            .await
            .expect("API server failed");
    });
    let proxy_server = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(proxy_addr)
            .await
            .expect("Failed to bind proxy server");
        axum::serve(listener, proxy_app)
            .with_graceful_shutdown(handle_shutdown())
            .await
            .expect("Proxy server failed");
    });

    let _ = tokio::join!(api_server, proxy_server);
}

async fn handle_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutting down...");
}
