use axum::extract::State;
use example_modular_model::{PetShower, ServiceContext};
use nano_rs::axum::errors::ServerError;
use nano_rs::axum::rest::RestResp;
use nano_rs::{biz_ok, get, post};

/// Give your Samoyed a bath
#[post(path = "/samoyed/shower", layers = ["example_modular_layers::auth::auth_token#{example_modular_model::ServiceContext}"])]
pub async fn shower(
    State(_svc): State<ServiceContext>,
) -> Result<RestResp<PetShower>, ServerError> {
    biz_ok!(PetShower {
        name: "mantou".to_string(),
        status: "clean".to_string(),
    })
}

/// Get Samoyed name
#[get(path = "/samoyed/name")]
pub async fn name() -> Result<RestResp<String>, ServerError> {
    biz_ok!("mantou".to_string())
}
