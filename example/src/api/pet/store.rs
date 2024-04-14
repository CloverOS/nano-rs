use axum::extract::State;
use nano_rs::axum::errors::ServerError;
use nano_rs::axum::rest::{biz_ok, RestResp};
use nano_rs::config::rest::RestConfig;
use nano_rs::get;

#[get(path = "/store/name", group = "Store", api = "Get the default pet store name", layers =
["crate::layers::auth::auth_token1"])]
pub async fn get_store_name() -> Result<RestResp<String>, ServerError> {
    biz_ok("Doggy Store".to_string())
}

#[get(path = "/store/tel", group = "Store", api = "Get Store's telephone number", layers = ["crate::layers::auth::auth_token#{crate::ServiceContext}"])]
pub async fn get_store_tel(State(rest_config): State<RestConfig>) -> Result<RestResp<String>, ServerError> {
    biz_ok(rest_config.name.to_string())
}