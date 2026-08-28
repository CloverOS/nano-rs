use axum::extract::State;
use example_modular_model::{Pet, ServiceContext};
use nano_rs::axum::errors::ServerError;
use nano_rs::axum::extractor::Json;
use nano_rs::axum::rest::RestResp;
use nano_rs::{biz_ok, post};

/// Add a new pet to the store(json)
#[post(path = "/store/pet/json")]
pub async fn handler(
    State(_svc): State<ServiceContext>,
    Json(pet): Json<Pet>,
) -> Result<RestResp<Pet>, ServerError> {
    biz_ok!(pet)
}
