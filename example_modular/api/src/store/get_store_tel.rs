use axum::extract::State;
use example_modular_model::ServiceContext;
use nano_rs::axum::errors::ServerError;
use nano_rs::axum::rest::RestResp;
use nano_rs::{biz_ok, get};

/// Get Store's telephone number
#[get(path = "/store/tel", layers = ["example_modular_layers::auth::auth_token#{example_modular_model::ServiceContext}"])]
pub async fn handler(State(svc): State<ServiceContext>) -> Result<RestResp<String>, ServerError> {
    biz_ok!(svc.rest_config.name.to_string())
}
