use nano_rs::axum::errors::ServerError;
use nano_rs::axum::rest::RestResp;
use nano_rs::{biz_ok, get};

/// Get store name
#[get(path = "/store/name", layers = ["example_modular_layers::auth::auth_token1"])]
pub async fn handler() -> Result<RestResp<String>, ServerError> {
    biz_ok!("Doggy Store".to_string())
}
