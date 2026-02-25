use example_modular_model::{Meta, Params, Pet};
use nano_rs::axum::errors::ServerError;
use nano_rs::axum::extractor::Path;
use nano_rs::axum::rest::RestResp;
use nano_rs::{biz_ok, get};

/// Get pet by id
#[get(path = "/store/pet/{id}")]
pub async fn handler(Path(params): Path<Params>) -> Result<RestResp<Pet>, ServerError> {
    biz_ok!(Pet {
        id: params.id,
        name: "Doggy".to_string(),
        tag: None,
        meta: Meta { age: 1 },
    })
}
