use axum::{Router};
use axum_client_ip;
use axum_client_ip::SecureClientIpSource;
use crate::axum::{handler, middleware};
use crate::axum::shutdown::shutdown_signal;

/// easy run axum server with rest config
///
/// # Example
/// ```rust
/// use axum::Router;
/// use nano_rs_extra::axum::start::run;
/// use nano_rs_core::config::rest::RestConfig;
/// use axum_client_ip::SecureClientIpSource;
///
/// #[tokio::main]
/// async fn main() {
///     let rest_config = nano_rs_core::config::init_config_with_cli::<RestConfig>();
///     let _guards = nano_rs_core::tracing::init_tracing(&rest_config);
///     let service_context = ServiceContext {
///         rest_config: rest_config.clone(),
///     };
///     let app = Router::new();
///     /// if use nginx proxy,you can use SecureClientIpSource::XRealIp
///     run(app, rest_config, SecureClientIpSource::XRealIp).await
/// }
///
/// #[derive(Clone)]
/// pub struct ServiceContext {
///     pub rest_config: RestConfig,
/// }
/// ```
pub async fn run(app: Router, service_config: nano_rs_core::config::rest::RestConfig, sci: SecureClientIpSource) {
    let log_request_body = service_config.log.enable_request_body_log.unwrap_or(true);
    let log_req = service_config.log.log_req.unwrap_or(true);
    let mut app = app.fallback(handler::not_page::handler_404);
    if log_request_body {
        app = app.route_layer(axum::middleware::from_fn(
            middleware::trace::trace_http_with_request_body,
        ));
    } else if log_req {
        app = app.route_layer(axum::middleware::from_fn(
            middleware::trace::trace_http,
        ));
    }
    app = app.layer(sci.into_extension());

    let host = service_config.host.unwrap_or_else(|| "127.0.0.1".to_string());
    let port = service_config.port.to_string();
    let address = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let link = format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, url);
    tracing::info!("listening on {}",link);
    axum::serve(listener, app.layer(tower_http::trace::TraceLayer::new_for_http()).into_make_service_with_connect_info::<std::net::SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await.unwrap();
}