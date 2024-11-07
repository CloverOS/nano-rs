use std::time::Instant;

use axum::body::{Body, Bytes};
use axum::extract::Request;
use axum::http::{Response, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum_client_ip::SecureClientIp;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};

pub async fn trace_http(
    SecureClientIp(secure_ip): SecureClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = Instant::now();

    let (method, path, ip) = (
        &req.method().to_string(),
        &req.uri().to_string(),
        secure_ip.to_string(),
    );
    let res = next.run(req).await;

    let duration = start.elapsed();
    tracing::info!(
        "method:{} path:{} ip:{} duration:{:?}",
        method,
        path,
        ip,
        duration
    );

    Ok(res)
}

pub async fn trace_http_with_request_body(
    secure_ip: SecureClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = Instant::now();
    let mut req_info = RequestInfo {
        method: String::from(&req.method().to_string()),
        path: String::from(&req.uri().to_string()),
        ip: secure_ip.0.to_string(),
        req_body: None,
        resp_body: None,
        duration: "".to_string(),
    };

    let (parts, body) = req.into_parts();
    let bytes = buffer_printer(body).await?;
    if let Ok(body) = std::str::from_utf8(&bytes) {
        req_info.req_body = Some(body.to_string());
    }
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let duration = start.elapsed();
    req_info.duration = format!("{:?}", duration);

    tracing::info!("{req_info}");
    Ok(res)
}

pub async fn trace_http_with_request_body_and_response_body(
    secure_ip: SecureClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = Instant::now();
    let mut req_info = RequestInfo {
        method: String::from(&req.method().to_string()),
        path: String::from(&req.uri().to_string()),
        ip: secure_ip.0.to_string(),
        req_body: None,
        resp_body: None,
        duration: "".to_string(),
    };

    let (parts, body) = req.into_parts();
    let bytes = buffer_printer(body).await?;
    if let Ok(body) = std::str::from_utf8(&bytes) {
        req_info.req_body = Some(body.to_string());
    }
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;
    let (parts, body) = res.into_parts();
    let bytes = buffer_printer(body).await?;
    if let Ok(body) = std::str::from_utf8(&bytes) {
        req_info.resp_body = Some(body.to_string());
    }
    let res = Response::from_parts(parts, Body::from(bytes));

    let duration = start.elapsed();
    req_info.duration = format!("{:?}", duration);

    tracing::info!("{req_info}");
    Ok(res)
}

pub async fn buffer_printer<B>(body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read body: {}", err),
            ));
        }
    };
    Ok(bytes)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestInfo {
    pub method: String,
    pub path: String,
    pub ip: String,
    pub req_body: Option<String>,
    pub resp_body: Option<String>,
    pub duration: String,
}

impl std::fmt::Display for RequestInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(json_str) => write!(f, "{}", json_str),
            Err(_) => Err(std::fmt::Error), // In case JSON Serialization fails
        }
    }
}
