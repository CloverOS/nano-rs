use axum::Router;
use axum_client_ip::ClientIpSource;
use example_modular_model::ServiceContext;
use nano_rs::axum::start::AppStarter;
use nano_rs::config::init_config_with_cli;
use nano_rs::config::rest::RestConfig;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

use crate::doc::GenApi;
use crate::routes::get_routes;

mod api_info;
mod doc;
mod routes;

#[tokio::main]
async fn main() {
    let rest_config = init_config_with_cli::<RestConfig>();
    let _guards = nano_rs::tracing::init_tracing(&rest_config);
    let service_context = ServiceContext {
        rest_config: rest_config.clone(),
    };

    let app = Router::new()
        .merge(RapiDoc::with_openapi("/api-docs/openapi.json", GenApi::openapi()).path("/doc"));
    let app = if rest_config.base_path.is_empty() || rest_config.base_path == "/" {
        app.merge(get_routes(service_context.clone()))
    } else {
        app.nest(
            rest_config.base_path.as_str(),
            get_routes(service_context.clone()),
        )
    };

    let app = app.layer(CorsLayer::new().allow_origin(Any).allow_methods(Any));

    AppStarter::new(app, rest_config.clone())
        .add_log_layer_with_config(Some(rest_config.log))
        .add_secure_client_ip_source_layer(ClientIpSource::ConnectInfo)
        .run()
        .await;
}
