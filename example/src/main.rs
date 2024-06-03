use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

use nano_rs::axum::start::run;
use nano_rs::config::init_config_with_cli;
use nano_rs::config::rest::RestConfig;

use crate::doc::GenApi;
use crate::routes::get_routes;

mod routes;
mod layers;
mod api;
mod types;
mod api_info;
mod model;
mod doc;

#[tokio::main]
async fn main() {
    let rest_config = init_config_with_cli::<RestConfig>();
    let _guards = nano_rs::tracing::init_tracing(&rest_config);
    let service_context = ServiceContext {
        rest_config: rest_config.clone(),
    };

    let app = Router::new()
        .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", GenApi::openapi()).path("/doc"))
        .nest(
            rest_config.base_path.clone().unwrap_or("".to_string()).as_str(),
            get_routes(service_context.clone(), rest_config.clone()),
        );
    let app = app.layer(CorsLayer::new()
                            .allow_origin(Any)
                            .allow_methods(Any), );
    run(app, rest_config).await
}


#[derive(Clone)]
pub struct ServiceContext {
    pub rest_config: RestConfig,
}