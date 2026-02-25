use axum::extract::{Query, State};
use axum::{Form, Json};
use example_modular_model::{Page, Params, Pet, PetForm, QueryPet, ServiceContext};
use nano_rs::axum::errors::ServerError;
use nano_rs::axum::extractor::Path;
use nano_rs::axum::rest::RestResp;
use nano_rs::{biz_err, biz_ok, get, post};

/// Get store name
#[get(path = "/store/name", layers = ["example_modular_layers::auth::auth_token1"])]
pub async fn get_store_name() -> Result<RestResp<String>, ServerError> {
    biz_ok!("Doggy Store".to_string())
}

/// Get Store's telephone number
#[get(path = "/store/tel", layers = ["example_modular_layers::auth::auth_token#{example_modular_model::ServiceContext}"])]
pub async fn get_store_tel(State(svc): State<ServiceContext>) -> Result<RestResp<String>, ServerError> {
    biz_ok!(svc.rest_config.name.to_string())
}

/// Add a new pet to the store(json)
#[post(path = "/store/pet/json")]
pub async fn add_json_pet(
    State(_svc): State<ServiceContext>,
    Json(pet): Json<Pet>,
) -> Result<RestResp<Pet>, ServerError> {
    biz_ok!(pet)
}

/// Add a new pet to the store(form)
#[post(path = "/store/pet/form")]
pub async fn add_form_pet(
    State(_svc): State<ServiceContext>,
    Form(_pet): Form<PetForm>,
) -> Result<RestResp<Pet>, ServerError> {
    biz_err!("failed")
}

/// Get pet by id
#[get(path = "/store/pet/{id}")]
pub async fn get_pet(Path(params): Path<Params>) -> Result<RestResp<Pet>, ServerError> {
    biz_ok!(Pet {
        id: params.id,
        name: "Doggy".to_string(),
        tag: None,
        meta: example_modular_model::Meta { age: 1 },
    })
}

/// Get pet list
#[post(path = "/store/pet/list/{page}/{count}")]
pub async fn pet_page_list(
    State(_svc): State<ServiceContext>,
    Path(_page): Path<Page>,
) -> Result<RestResp<Vec<Pet>>, ServerError> {
    biz_ok!(vec![])
}

/// Query pet by id
#[get(path = "/store/pet")]
pub async fn query_pet(
    Query(query): Query<QueryPet>,
) -> Result<RestResp<Pet>, ServerError> {
    biz_ok!(Pet {
        id: query.id,
        name: "Doggy".to_string(),
        tag: None,
        meta: example_modular_model::Meta { age: 1 },
    })
}
