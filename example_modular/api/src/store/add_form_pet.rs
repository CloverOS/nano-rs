use axum::extract::State;
use axum::Form;
use example_modular_model::{Pet, PetForm, ServiceContext};
use nano_rs::axum::errors::ServerError;
use nano_rs::axum::rest::RestResp;
use nano_rs::{biz_err, post};

/// Add a new pet to the store(form)
#[post(path = "/store/pet/form")]
pub async fn handler(
    State(_svc): State<ServiceContext>,
    Form(_pet): Form<PetForm>,
) -> Result<RestResp<Pet>, ServerError> {
    biz_err!("failed")
}
