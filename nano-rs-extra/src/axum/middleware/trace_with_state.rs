//! Access-log middleware with shared, read-only logging configuration.
//!
//! These entry points require `Arc<LogConfig>` as middleware state. Direct callers
//! migrating from `State<LogConfig>` must wrap the configuration once before
//! mounting the middleware:
//!
//! ```
//! use std::sync::Arc;
//! use axum::{Router, middleware::from_fn_with_state, routing::get};
//! use axum_client_ip::ClientIpSource;
//! use nano_rs_core::config::logger::LogConfig;
//! use nano_rs_extra::axum::middleware::trace_with_state::trace_http_with_state;
//!
//! let config = Arc::new(LogConfig::default());
//! let app: Router = Router::new()
//!     .route("/", get(|| async { "ok" }))
//!     .route_layer(from_fn_with_state(config, trace_http_with_state))
//!     .layer(ClientIpSource::XRealIp.into_extension());
//! ```

use std::sync::Arc;

use crate::axum::middleware::trace::{TraceMode, run_trace_core};
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum_client_ip::ClientIp;
use nano_rs_core::config::logger::LogConfig;

pub async fn trace_http_with_state(
    State(log_config): State<Arc<LogConfig>>,
    secure_ip: ClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    run_trace_core(
        Some(log_config.as_ref()),
        secure_ip,
        req,
        next,
        TraceMode::BASIC,
    )
    .await
}

pub async fn trace_http_with_request_body_with_state(
    State(log_config): State<Arc<LogConfig>>,
    secure_ip: ClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    run_trace_core(
        Some(log_config.as_ref()),
        secure_ip,
        req,
        next,
        TraceMode::REQUEST_BODY,
    )
    .await
}

pub async fn trace_http_with_request_body_and_response_body_with_state(
    State(log_config): State<Arc<LogConfig>>,
    secure_ip: ClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    run_trace_core(
        Some(log_config.as_ref()),
        secure_ip,
        req,
        next,
        TraceMode::REQUEST_AND_RESPONSE_BODY,
    )
    .await
}
