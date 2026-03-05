use crate::axum::middleware::trace::{TraceMode, run_trace_core};
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum_client_ip::ClientIp;
use nano_rs_core::config::logger::LogConfig;

pub async fn trace_http_with_state(
    State(log_config): State<LogConfig>,
    secure_ip: ClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    run_trace_core(Some(&log_config), secure_ip, req, next, TraceMode::BASIC).await
}

pub async fn trace_http_with_request_body_with_state(
    State(log_config): State<LogConfig>,
    secure_ip: ClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    run_trace_core(
        Some(&log_config),
        secure_ip,
        req,
        next,
        TraceMode::REQUEST_BODY,
    )
    .await
}

pub async fn trace_http_with_request_body_and_response_body_with_state(
    State(log_config): State<LogConfig>,
    secure_ip: ClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    run_trace_core(
        Some(&log_config),
        secure_ip,
        req,
        next,
        TraceMode::REQUEST_AND_RESPONSE_BODY,
    )
    .await
}
