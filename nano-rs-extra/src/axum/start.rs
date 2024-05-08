use axum::{Router};
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
///
/// #[tokio::main]
/// async fn main() {
///     let rest_config = nano_rs_core::config::init_config_with_cli::<RestConfig>();
///     let _guard = nano_rs_core::tracing::init_tracing(&rest_config);
///     let service_context = ServiceContext {
///         rest_config: rest_config.clone(),
///     };
///     let app = Router::new();
///     run(app, rest_config).await
/// }
///
/// #[derive(Clone)]
/// pub struct ServiceContext {
///     pub rest_config: RestConfig,
/// }
/// ```
pub async fn run(app: Router, service_config: nano_rs_core::config::rest::RestConfig) {
    let app = match service_config.logger.enable_request_body_log {
        true => {
            app
                .fallback(handler::not_page::handler_404)
                .route_layer(axum::middleware::from_fn(
                    middleware::trace::trace_http_with_request_body,
                ))
                .layer(SecureClientIpSource::ConnectInfo.into_extension())
        }
        false => {
            app
                .fallback(handler::not_page::handler_404)
                .route_layer(axum::middleware::from_fn(
                    middleware::trace::trace_http,
                ))
                .layer(SecureClientIpSource::ConnectInfo.into_extension())
        }
    };
    let host: String;
    if service_config.host.is_some() {
        host = service_config.host.unwrap();
    } else {
        host = "127.0.0.1".to_string();
    }
    let listener = tokio::net::TcpListener::bind(host + ":" + service_config.port.to_string().as_str())
        .await
        .unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let link = format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, url);
    tracing::info!("listening on {}",link);
    axum::serve(listener,
                app.layer(tower_http::trace::TraceLayer::new_for_http())
                    .into_make_service_with_connect_info::<std::net::SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await.unwrap();
}