use axum::Router;
use nano_rs::axum::start::run;
use nano_rs::config::init_config_with_cli;
use nano_rs::config::rest::RestConfig;
use crate::routes::get_routes;

mod routes;
mod layers;
mod api;
mod types;
mod api_info;

#[tokio::main]
async fn main() {
    let rest_config = init_config_with_cli::<RestConfig>();
    let _guard = nano_rs::tracing::init_tracing(&rest_config);
    let service_context = ServiceContext {
        rest_config: rest_config.clone(),
    };
    let app = Router::new().nest(
        rest_config.base_path.as_str(),
        get_routes(service_context.clone(), rest_config.clone()),
    );
    run(app, rest_config).await
}

#[derive(Clone)]
pub struct ServiceContext {
    pub rest_config: RestConfig,
}