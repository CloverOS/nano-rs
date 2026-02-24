use crate::axum::middleware::trace::{
    RequestInfo, buffer_printer, resolve_request_id, set_request_id_header,
};
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{Response, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum_client_ip::ClientIp;
use nano_rs_core::config::logger::LogConfig;
use std::time::Instant;

/// ignore trace
fn should_ignore_trace(log_config: &LogConfig, method: &str, path: &str) -> bool {
    if log_config.ignore_resource.is_none() {
        return false;
    }
    log_config
        .ignore_resource
        .as_ref()
        .map(|resources| {
            resources
                .iter()
                .any(|resource| resource.method == method && resource.path == path)
        })
        .unwrap_or(false)
}

pub async fn trace_http_with_state(
    State(log_config): State<LogConfig>,
    ClientIp(secure_ip): ClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let request_id = resolve_request_id(req.headers());
    if should_ignore_trace(
        &log_config,
        &req.method().to_string(),
        &req.uri().path().to_string(),
    ) {
        let mut res = next.run(req).await;
        set_request_id_header(res.headers_mut(), &request_id);
        return Ok(res);
    }
    let start = Instant::now();

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

pub async fn trace_http_with_request_body_with_state(
    State(log_config): State<LogConfig>,
    secure_ip: ClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let request_id = resolve_request_id(req.headers());
    if should_ignore_trace(
        &log_config,
        &req.method().to_string(),
        &req.uri().path().to_string(),
    ) {
        let mut res = next.run(req).await;
        set_request_id_header(res.headers_mut(), &request_id);
        return Ok(res);
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

    let mut res = next.run(req).await;
    set_request_id_header(res.headers_mut(), &request_id);

    let duration = start.elapsed();
    req_info.duration = format!("{:?}", duration);

    tracing::info!("{req_info}");
    Ok(res)
}

pub async fn trace_http_with_request_body_and_response_body_with_state(
    State(log_config): State<LogConfig>,
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
    if should_ignore_trace(
        &log_config,
        &req.method().to_string(),
        &req.uri().path().to_string(),
    ) {
        let mut res = next.run(req).await;
        set_request_id_header(res.headers_mut(), &request_id);
        return Ok(res);
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
