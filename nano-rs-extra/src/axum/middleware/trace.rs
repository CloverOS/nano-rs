use std::time::Instant;

use axum::body::{Body, Bytes};
use axum::extract::Request;
use axum::http::StatusCode;
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

    let (method, path, ip) = (&req.method().to_string(), &req.uri().to_string(), secure_ip.to_string());
    let res = next.run(req).await;

    let duration = start.elapsed();
    tracing::info!("method:{} path:{} ip:{} duration:{:?}",method,path,ip,duration);

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
        req_body: "".to_string(),
        duration: "".to_string(),
    };

    let (parts, body) = req.into_parts();
    let bytes = req_log(body, &mut req_info).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let duration = start.elapsed();
    req_info.duration = format!("{:?}", duration);

    tracing::info!("{req_info}");
    Ok(res)
}

pub async fn req_log<B>(body: B, request_info: &mut RequestInfo) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data=Bytes>,
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

    if let Ok(body) = std::str::from_utf8(&bytes) {
        request_info.req_body = body.to_string();
    }

    Ok(bytes)
}


#[derive(Debug, Deserialize, Serialize)]
pub struct RequestInfo {
    pub method: String,
    pub path: String,
    pub ip: String,
    pub req_body: String,
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
