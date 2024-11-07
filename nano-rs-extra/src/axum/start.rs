use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::Request;
use axum::response::IntoResponse;
use axum::routing::Route;
use axum::Router;
use axum_client_ip;
use axum_client_ip::SecureClientIpSource;
use tower::{Layer, Service};
use tower_http::cors::{Any, CorsLayer};

use nano_rs_core::config::rest::RestConfig;

use crate::axum::shutdown::shutdown_signal;
use crate::axum::{handler, middleware};

/// AppStarter
pub struct AppStarter {
    pub app: Router,
    pub rest_config: Arc<RestConfig>,
}

impl AppStarter {
    /// new starter with axum app and rest config
    pub fn new(app: Router, rest_config: RestConfig) -> Self {
        AppStarter {
            app,
            rest_config: Arc::new(rest_config),
        }
    }

    /// easy run axum server with rest config
    ///
    /// # Example
    /// ```rust
    /// use axum::Router;
    /// use nano_rs_core::config::rest::RestConfig;
    /// use axum_client_ip::SecureClientIpSource;
    /// use nano_rs_extra::axum::start::AppStarter;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let rest_config = nano_rs_core::config::init_config_with_cli::<RestConfig>();
    ///     let _guards = nano_rs_core::tracing::init_tracing(&rest_config);
    ///     let service_context = ServiceContext {
    ///         rest_config: rest_config.clone(),
    ///     };
    ///     let app = Router::new();
    ///     AppStarter::new(app, rest_config)
    ///         .add_log_layer()
    ///         .add_secure_client_ip_source_layer(SecureClientIpSource::XRealIp)
    ///         .run()
    ///         .await;
    /// }
    ///
    /// #[derive(Clone)]
    /// pub struct ServiceContext {
    ///     pub rest_config: RestConfig,
    /// }
    /// ```
    pub async fn run(self) {
        let host = self
            .rest_config
            .host
            .clone()
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let port = self.rest_config.port.clone().to_string();
        let address = format!("{}:{}", host, port);
        let listener = tokio::net::TcpListener::bind(address).await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let link = format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, url);
        tracing::info!("listening on {}", link);
        axum::serve(
            listener,
            self.app
                .into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    }

    /// run service with dev mode(all cors allowed)
    /// # Example
    /// ```rust
    /// use axum::Router;
    /// use nano_rs_core::config::rest::RestConfig;
    /// use axum_client_ip::SecureClientIpSource;
    /// use nano_rs_extra::axum::start::AppStarter;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let rest_config = nano_rs_core::config::init_config_with_cli::<RestConfig>();
    ///     let _guards = nano_rs_core::tracing::init_tracing(&rest_config);
    ///     let service_context = ServiceContext {
    ///         rest_config: rest_config.clone(),
    ///     };
    ///     let app = Router::new();
    ///     AppStarter::new(app,rest_config).run_dev().await;
    /// }
    ///
    /// #[derive(Clone)]
    /// pub struct ServiceContext {
    ///     pub rest_config: RestConfig,
    /// }
    /// ```
    pub async fn run_dev(self) {
        self.add_log_layer()
            .add_secure_client_ip_source_layer(SecureClientIpSource::ConnectInfo)
            .add_dev_cors_layer()
            .run()
            .await;
    }

    /// add log layer to axum app
    pub fn add_log_layer(mut self) -> Self {
        let log_request_body = self
            .rest_config
            .log
            .enable_request_body_log
            .clone()
            .unwrap_or(true);
        let log_response_body = self
            .rest_config
            .log
            .enable_response_body_log
            .clone()
            .unwrap_or(false);
        let log_req = self.rest_config.log.log_req.clone().unwrap_or(true);
        let app = self.app.clone().fallback(handler::not_page::handler_404);
        self.app = if log_req {
            if log_request_body {
                if log_response_body {
                    app.route_layer(axum::middleware::from_fn(
                        middleware::trace::trace_http_with_request_body_and_response_body,
                    ))
                } else {
                    app.route_layer(axum::middleware::from_fn(
                        middleware::trace::trace_http_with_request_body,
                    ))
                }
            } else {
                app.route_layer(axum::middleware::from_fn(middleware::trace::trace_http))
            }
        } else {
            self.app
        };
        self
    }

    /// add secure client ip source layer to axum app
    pub fn add_secure_client_ip_source_layer(mut self, sci: SecureClientIpSource) -> Self {
        self.app = self.app.layer(sci.into_extension());
        self
    }

    /// add all allowed cors layer to axum app(dev mode)
    pub fn add_dev_cors_layer(mut self) -> Self {
        self.app = self.app.layer(
            CorsLayer::new()
                .allow_headers(Any)
                .allow_origin(Any)
                .allow_methods(Any),
        );
        self
    }

    /// add trace layer to axum app
    pub fn add_trace_layer(mut self) -> Self {
        self.app = self
            .app
            .layer(tower_http::trace::TraceLayer::new_for_http());
        self
    }

    /// add layer to axum app
    pub fn add_layer<L>(mut self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + 'static,
        L::Service: Service<Request> + Clone + Send + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        self.app = self.app.layer(layer);
        self
    }
}
