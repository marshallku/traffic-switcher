use axum::{
    body::Body,
    http::{header, HeaderValue, Response, StatusCode},
};
use mime_guess::mime;
use percent_encoding::percent_decode_str;
use std::path::PathBuf;
use tokio::fs;
use tracing::error;

pub async fn serve_static_file(
    root: &str,
    path: &str,
    index_files: &[String],
    try_files: &[String],
) -> Result<Response<Body>, StatusCode> {
    let decoded_path = percent_decode_str(path)
        .decode_utf8()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut file_path = PathBuf::from(root);

    for segment in decoded_path.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." || segment.contains('\0') {
            return Err(StatusCode::FORBIDDEN);
        }
        file_path.push(segment);
    }

    if file_path.is_dir() {
        for index in index_files.iter() {
            let index_path = file_path.join(index);
            if index_path.exists() && index_path.is_file() {
                file_path = index_path;
                break;
            }
        }
    }

    if !file_path.exists() && !try_files.is_empty() {
        for try_file in try_files {
            let try_path = if try_file.starts_with('/') {
                PathBuf::from(root).join(&try_file[1..])
            } else {
                file_path.parent().unwrap_or(&file_path).join(try_file)
            };

            if try_path.exists() && try_path.is_file() {
                file_path = try_path;
                break;
            }
        }
    }

    if !file_path.exists() || !file_path.is_file() {
        return Err(StatusCode::NOT_FOUND);
    }

    let contents = fs::read(&file_path).await.map_err(|e| {
        error!("Failed to read file {:?}: {}", file_path, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mime_type = mime_guess::from_path(&file_path)
        .first()
        .unwrap_or(mime::APPLICATION_OCTET_STREAM);

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type.as_ref())
        .body(Body::from(contents))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if mime_type.type_() == mime::TEXT || mime_type == mime::APPLICATION_JAVASCRIPT {
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&format!("{}; charset=utf-8", mime_type))
                .unwrap_or_else(|_| HeaderValue::from_str(mime_type.as_ref()).unwrap()),
        );
    }

    Ok(response)
}
