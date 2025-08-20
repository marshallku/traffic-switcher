use axum::{
    extract::{Host, Request, State},
    http::{uri::Uri, StatusCode},
    response::{IntoResponse, Response},
};
use hyper::client::conn::http1::Builder;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use tracing::error;

use crate::env::state::{AppState, RouteTarget};
use crate::routes::static_files::serve_static_file;

pub async fn proxy_handler(
    Host(host): Host,
    State(state): State<AppState>,
    req: Request,
) -> Result<Response, StatusCode> {
    let routes = state.routes_map.read().await;
    let domain = host.split(':').next().unwrap_or(&host);

    let route_target = routes
        .get(domain)
        .or_else(|| routes.get("*"))
        .ok_or(StatusCode::NOT_FOUND)?;

    match route_target {
        RouteTarget::Service { service } => {
            let services = state.services_map.read().await;
            let service_config = services.get(service).ok_or(StatusCode::BAD_GATEWAY)?;
            let target_addr = format!("{}:{}", service_config.host, service_config.port);
            proxy_request(req, &target_addr).await
        }
        RouteTarget::Static {
            root,
            index,
            try_files,
        } => {
            let path = req.uri().path();
            let default_index = vec!["index.html".to_string()];
            let index_files = if index.is_empty() {
                &default_index
            } else {
                index
            };

            serve_static_file(root, path, index_files, try_files)
                .await
                .map(|res| res.into_response())
        }
    }
}

pub async fn proxy_request(mut req: Request, target: &str) -> Result<Response, StatusCode> {
    let stream = TcpStream::connect(target).await.map_err(|e| {
        error!("Failed to connect to {}: {}", target, e);
        StatusCode::BAD_GATEWAY
    })?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = Builder::new()
        .preserve_header_case(true)
        .title_case_headers(true)
        .handshake(io)
        .await
        .map_err(|e| {
            error!("Handshake error: {}", e);
            StatusCode::BAD_GATEWAY
        })?;

    tokio::spawn(async move {
        if let Err(err) = conn.await {
            error!("Connection error: {}", err);
        }
    });

    let uri = req.uri();
    let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");

    *req.uri_mut() = path_and_query
        .parse::<Uri>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let response = sender.send_request(req).await.map_err(|e| {
        error!("Request error: {}", e);
        StatusCode::BAD_GATEWAY
    })?;

    Ok(response.into_response())
}
