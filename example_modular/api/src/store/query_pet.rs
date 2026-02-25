use axum::extract::Query;
use example_modular_model::{Meta, Pet, QueryPet};
use nano_rs::axum::errors::ServerError;
use nano_rs::axum::rest::RestResp;
use nano_rs::{biz_ok, get};

/// Query pet by id
#[get(path = "/store/pet")]
pub async fn handler(Query(query): Query<QueryPet>) -> Result<RestResp<Pet>, ServerError> {
    biz_ok!(Pet {
        id: query.id,
        name: "Doggy".to_string(),
        tag: None,
        meta: Meta { age: 1 },
    })
}
