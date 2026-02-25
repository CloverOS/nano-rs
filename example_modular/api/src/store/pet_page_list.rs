use axum::extract::State;
use example_modular_model::{Page, Pet, ServiceContext};
use nano_rs::axum::errors::ServerError;
use nano_rs::axum::extractor::Path;
use nano_rs::axum::rest::RestResp;
use nano_rs::{biz_ok, post};

/// Get pet list
#[post(path = "/store/pet/list/{page}/{count}")]
pub async fn handler(
    State(_svc): State<ServiceContext>,
    Path(_page): Path<Page>,
) -> Result<RestResp<Vec<Pet>>, ServerError> {
    biz_ok!(vec![])
}
