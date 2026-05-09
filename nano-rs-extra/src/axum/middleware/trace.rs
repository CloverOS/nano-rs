use std::time::Instant;

use axum::body::{Body, Bytes};
use axum::extract::{MatchedPath, Request};
use axum::http::header::HeaderValue;
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum_client_ip::ClientIp;
use http_body_util::BodyExt;
use nano_rs_core::config::logger::LogConfig;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub(crate) const REQUEST_ID_HEADER: &str = "x-request-id";

#[derive(Clone, Copy, Debug)]
pub(crate) struct TraceMode {
    log_request_body: bool,
    log_response_body: bool,
}

impl TraceMode {
    pub(crate) const BASIC: Self = Self {
        log_request_body: false,
        log_response_body: false,
    };

    pub(crate) const REQUEST_BODY: Self = Self {
        log_request_body: true,
        log_response_body: false,
    };

    pub(crate) const REQUEST_AND_RESPONSE_BODY: Self = Self {
        log_request_body: true,
        log_response_body: true,
    };

    fn uses_body_log(self) -> bool {
        self.log_request_body || self.log_response_body
    }
}

pub async fn trace_http(
    secure_ip: ClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    run_trace_core(None, secure_ip, req, next, TraceMode::BASIC).await
}

pub async fn trace_http_with_request_body(
    secure_ip: ClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    run_trace_core(None, secure_ip, req, next, TraceMode::REQUEST_BODY).await
}

pub async fn trace_http_with_request_body_and_response_body(
    secure_ip: ClientIp,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    run_trace_core(
        None,
        secure_ip,
        req,
        next,
        TraceMode::REQUEST_AND_RESPONSE_BODY,
    )
    .await
}

pub(crate) async fn run_trace_core(
    log_config: Option<&LogConfig>,
    secure_ip: ClientIp,
    req: Request,
    next: Next,
    mode: TraceMode,
) -> Result<Response, (StatusCode, String)> {
    let start = Instant::now();
    let request_id = resolve_request_id(req.headers());
    let method = req.method().to_string();
    let path = req.uri().to_string();
    let actual_path = req.uri().path();
    let matched_path = req
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str);
    let ip = secure_ip.0.to_string();

    if should_ignore_trace(log_config, method.as_str(), actual_path, matched_path) {
        return Ok(early_return_with_request_id(next, req, &request_id).await);
    }

    let mut req_info = mode.uses_body_log().then(|| RequestInfo {
        request_id: request_id.clone(),
        method: method.clone(),
        path: path.clone(),
        ip: ip.clone(),
        req_body: None,
        resp_body: None,
        duration: String::new(),
    });

    let req = if mode.log_request_body {
        let (parts, body) = req.into_parts();
        let bytes = buffer_printer(body).await?;
        if let Ok(body) = std::str::from_utf8(&bytes) {
            if let Some(info) = req_info.as_mut() {
                info.req_body = Some(body.to_string());
            }
        }
        Request::from_parts(parts, Body::from(bytes))
    } else {
        req
    };

    if is_websocket_upgrade_request(req.headers()) {
        log_trace(start, &request_id, &method, &path, &ip, req_info.as_mut());
        return Ok(early_return_with_request_id(next, req, &request_id).await);
    }

    if mode.log_response_body && is_sse_request(req.headers()) {
        log_trace(start, &request_id, &method, &path, &ip, req_info.as_mut());
        return Ok(early_return_with_request_id(next, req, &request_id).await);
    }

    let mut res = next.run(req).await;

    if mode.log_response_body && is_sse_response(res.headers()) {
        set_request_id_header(res.headers_mut(), &request_id);
        log_trace(start, &request_id, &method, &path, &ip, req_info.as_mut());
        return Ok(res);
    }

    if mode.log_response_body {
        let (parts, body) = res.into_parts();
        let bytes = buffer_printer(body).await?;
        if let Ok(body) = std::str::from_utf8(&bytes) {
            if let Some(info) = req_info.as_mut() {
                info.resp_body = Some(body.to_string());
            }
        }
        res = Response::from_parts(parts, Body::from(bytes));
    }

    set_request_id_header(res.headers_mut(), &request_id);
    log_trace(start, &request_id, &method, &path, &ip, req_info.as_mut());

    Ok(res)
}

fn log_trace(
    start: Instant,
    request_id: &str,
    method: &str,
    path: &str,
    ip: &str,
    req_info: Option<&mut RequestInfo>,
) {
    let duration = start.elapsed();

    if let Some(req_info) = req_info {
        req_info.duration = format!("{:?}", duration);
        tracing::info!("{req_info}");
        return;
    }

    tracing::info!(
        "request_id:{} method:{} path:{} ip:{} duration:{:?}",
        request_id,
        method,
        path,
        ip,
        duration
    );
}

pub(crate) async fn early_return_with_request_id(
    next: Next,
    req: Request,
    request_id: &str,
) -> Response {
    let mut res = next.run(req).await;
    set_request_id_header(res.headers_mut(), request_id);
    res
}

pub(crate) fn should_ignore_trace(
    log_config: Option<&LogConfig>,
    method: &str,
    actual_path: &str,
    matched_path: Option<&str>,
) -> bool {
    log_config
        .and_then(|config| config.ignore_resource.as_ref())
        .map(|resources| {
            resources.iter().any(|resource| {
                resource.method == method
                    && (resource.path == actual_path
                        || matched_path
                            .map(|path| resource.path == path)
                            .unwrap_or(false))
            })
        })
        .unwrap_or(false)
}

pub(crate) fn is_websocket_upgrade_request(headers: &HeaderMap) -> bool {
    headers
        .get(header::UPGRADE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
}

pub(crate) fn is_sse_request(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_ascii_lowercase().contains("text/event-stream"))
        .unwrap_or(false)
}

pub(crate) fn is_sse_response(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_ascii_lowercase().contains("text/event-stream"))
        .unwrap_or(false)
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
            Err(_) => Err(std::fmt::Error),
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

#[cfg(test)]
mod tests {
    use super::*;

    use axum::Router;
    use axum::http::{Request, StatusCode, header};
    use axum::middleware::{from_fn, from_fn_with_state};
    use axum::response::Response;
    use axum::routing::{get, post};
    use axum_client_ip::ClientIpSource;
    use futures_util::stream;
    use nano_rs_core::config::logger::{LogConfig, Resource};
    use tower::ServiceExt;

    use std::collections::HashMap;
    use std::io;
    use std::time::Duration;

    #[test]
    fn is_sse_response_matches_plain_content_type() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/event-stream"),
        );

        assert!(is_sse_response(&headers));
    }

    #[test]
    fn is_sse_response_matches_content_type_with_params() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/event-stream; charset=utf-8"),
        );

        assert!(is_sse_response(&headers));
    }

    #[test]
    fn is_sse_response_matches_mixed_case() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("Text/Event-Stream"),
        );

        assert!(is_sse_response(&headers));
    }

    #[tokio::test]
    async fn non_sse_response_is_buffered_and_request_id_is_set() {
        let app = Router::new()
            .route("/echo", post(|body: String| async move { body }))
            .route_layer(from_fn(trace_http_with_request_body_and_response_body))
            .layer(ClientIpSource::XRealIp.into_extension());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/echo")
                    .header("x-real-ip", "127.0.0.1")
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(Body::from("hello"))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key(REQUEST_ID_HEADER));

        let body = response
            .into_body()
            .collect()
            .await
            .expect("collect body")
            .to_bytes();
        assert_eq!(body, Bytes::from_static(b"hello"));
    }

    #[tokio::test]
    async fn sse_accept_request_is_not_buffered() {
        let app = Router::new()
            .route(
                "/stream",
                get(|| async {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/plain")
                        .body(Body::from_stream(
                            stream::pending::<Result<Bytes, io::Error>>(),
                        ))
                        .expect("response")
                }),
            )
            .route_layer(from_fn(trace_http_with_request_body_and_response_body))
            .layer(ClientIpSource::XRealIp.into_extension());

        let request = Request::builder()
            .method("GET")
            .uri("/stream")
            .header("x-real-ip", "127.0.0.1")
            .header(header::ACCEPT, "text/event-stream")
            .body(Body::empty())
            .expect("request");

        let response = tokio::time::timeout(Duration::from_millis(300), app.oneshot(request))
            .await
            .expect("middleware should not block on buffering")
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key(REQUEST_ID_HEADER));
    }

    #[tokio::test]
    async fn sse_response_content_type_is_not_buffered() {
        let app = Router::new()
            .route(
                "/stream",
                get(|| async {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/event-stream")
                        .body(Body::from_stream(
                            stream::pending::<Result<Bytes, io::Error>>(),
                        ))
                        .expect("response")
                }),
            )
            .route_layer(from_fn(trace_http_with_request_body_and_response_body))
            .layer(ClientIpSource::XRealIp.into_extension());

        let request = Request::builder()
            .method("GET")
            .uri("/stream")
            .header("x-real-ip", "127.0.0.1")
            .body(Body::empty())
            .expect("request");

        let response = tokio::time::timeout(Duration::from_millis(300), app.oneshot(request))
            .await
            .expect("middleware should not block on SSE responses")
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key(REQUEST_ID_HEADER));
    }

    #[tokio::test]
    async fn with_state_ignore_resource_short_circuits_buffering() {
        let log_config = LogConfig {
            ignore_resource: Some(vec![Resource {
                method: "POST".to_string(),
                path: "/ignore".to_string(),
            }]),
            logging: HashMap::new(),
            ..Default::default()
        };

        let app = Router::new()
            .route(
                "/ignore",
                post(|| async {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/plain")
                        .body(Body::from_stream(
                            stream::pending::<Result<Bytes, io::Error>>(),
                        ))
                        .expect("response")
                }),
            )
            .route_layer(from_fn_with_state(
                log_config,
                crate::axum::middleware::trace_with_state::trace_http_with_request_body_and_response_body_with_state,
            ))
            .layer(ClientIpSource::XRealIp.into_extension());

        let request = Request::builder()
            .method("POST")
            .uri("/ignore")
            .header("x-real-ip", "127.0.0.1")
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from("payload"))
            .expect("request");

        let response = tokio::time::timeout(Duration::from_millis(300), app.oneshot(request))
            .await
            .expect("ignore_resource should bypass buffering")
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key(REQUEST_ID_HEADER));
    }

    #[tokio::test]
    async fn with_state_ignore_resource_matches_axum_route_template() {
        let log_config = LogConfig {
            ignore_resource: Some(vec![Resource {
                method: "POST".to_string(),
                path: "/v1/create/{id}".to_string(),
            }]),
            logging: HashMap::new(),
            ..Default::default()
        };

        let app = Router::new()
            .route(
                "/v1/create/{id}",
                post(|| async {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/plain")
                        .body(Body::from_stream(
                            stream::pending::<Result<Bytes, io::Error>>(),
                        ))
                        .expect("response")
                }),
            )
            .route_layer(from_fn_with_state(
                log_config,
                crate::axum::middleware::trace_with_state::trace_http_with_request_body_and_response_body_with_state,
            ))
            .layer(ClientIpSource::XRealIp.into_extension());

        let request = Request::builder()
            .method("POST")
            .uri("/v1/create/123")
            .header("x-real-ip", "127.0.0.1")
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from("payload"))
            .expect("request");

        let response = tokio::time::timeout(Duration::from_millis(300), app.oneshot(request))
            .await
            .expect("ignore_resource should match the axum route template and bypass buffering")
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key(REQUEST_ID_HEADER));
    }

    #[tokio::test]
    async fn websocket_upgrade_request_is_not_buffered() {
        let app = Router::new()
            .route(
                "/ws",
                get(|| async {
                    Response::builder()
                        .status(StatusCode::SWITCHING_PROTOCOLS)
                        .header(header::CONTENT_TYPE, "text/plain")
                        .body(Body::from_stream(
                            stream::pending::<Result<Bytes, io::Error>>(),
                        ))
                        .expect("response")
                }),
            )
            .route_layer(from_fn(trace_http_with_request_body_and_response_body))
            .layer(ClientIpSource::XRealIp.into_extension());

        let request = Request::builder()
            .method("GET")
            .uri("/ws")
            .header("x-real-ip", "127.0.0.1")
            .header(header::UPGRADE, "websocket")
            .body(Body::empty())
            .expect("request");

        let response = tokio::time::timeout(Duration::from_millis(300), app.oneshot(request))
            .await
            .expect("websocket upgrade should bypass buffering")
            .expect("response");

        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
        assert!(response.headers().contains_key(REQUEST_ID_HEADER));
    }
}
