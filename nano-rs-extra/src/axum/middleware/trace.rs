use std::time::Instant;

use axum::body::{Body, Bytes};
use axum::extract::Request;
use axum::http::header::HeaderValue;
use axum::http::{HeaderMap, Response, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum_client_ip::ClientIp;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub(crate) const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn trace_http(
    ClientIp(secure_ip): ClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = Instant::now();
    let request_id = resolve_request_id(req.headers());

    let (method, path, ip) = (
        &req.method().to_string(),
        &req.uri().to_string(),
        secure_ip.to_string(),
    );
    let mut res = next.run(req).await;
    set_request_id_header(res.headers_mut(), &request_id);

    let duration = start.elapsed();
    tracing::info!(
        "request_id:{} method:{} path:{} ip:{} duration:{:?}",
        request_id,
        method,
        path,
        ip,
        duration
    );

    Ok(res)
}

pub async fn trace_http_with_request_body(
    secure_ip: ClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = Instant::now();
    let request_id = resolve_request_id(req.headers());
    let mut req_info = RequestInfo {
        request_id: request_id.clone(),
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

    let mut res = next.run(req).await;
    set_request_id_header(res.headers_mut(), &request_id);

    let duration = start.elapsed();
    req_info.duration = format!("{:?}", duration);

    tracing::info!("{req_info}");
    Ok(res)
}

pub async fn trace_http_with_request_body_and_response_body(
    secure_ip: ClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let request_id = resolve_request_id(req.headers());
    // 检查WebSocket升级请求
    if let Some(upgrade) = req.headers().get("upgrade") {
        if let Ok(upgrade_str) = upgrade.to_str() {
            if upgrade_str.eq_ignore_ascii_case("websocket") {
                let mut res = next.run(req).await;
                set_request_id_header(res.headers_mut(), &request_id);
                return Ok(res);
            }
        }
    }

    // 检查SSE请求 (Accept: text/event-stream)
    if let Some(accept) = req.headers().get("accept") {
        if let Ok(accept_str) = accept.to_str() {
            if accept_str.contains("text/event-stream") {
                let mut res = next.run(req).await;
                set_request_id_header(res.headers_mut(), &request_id);
                return Ok(res);
            }
        }
    }
    let start = Instant::now();
    let mut req_info = RequestInfo {
        request_id: request_id.clone(),
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
    let mut res = Response::from_parts(parts, Body::from(bytes));
    set_request_id_header(res.headers_mut(), &request_id);

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
    pub request_id: String,
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

pub(crate) fn resolve_request_id(headers: &HeaderMap) -> String {
    headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

pub(crate) fn set_request_id_header(headers: &mut HeaderMap, request_id: &str) {
    if let Ok(value) = HeaderValue::from_str(request_id) {
        headers.insert(REQUEST_ID_HEADER, value);
    }
}
