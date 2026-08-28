use axum::Form;
use axum::extract::{Query, State};

use nano_rs::axum::errors::ServerError;
use nano_rs::axum::extractor::{Json, Path};
use nano_rs::axum::rest::RestResp;
use nano_rs::config::rest::RestConfig;
use nano_rs::{biz_err, biz_ok, get, post};

use crate::model::pet::{Meta, Page, Params, Pet, PetForm, QueryPet};

/// Get the default pet store name
#[get(path = "/store/name", layers = ["crate::layers::auth::auth_token1"])]
pub async fn get_store_name() -> Result<RestResp<String>, ServerError> {
    biz_ok!("Doggy Store".to_string())
}

/// Get Store's telephone number
#[get(path = "/store/tel", layers = ["crate::layers::auth::auth_token#{crate::ServiceContext}"])]
pub async fn get_store_tel(
    State(rest_config): State<RestConfig>,
) -> Result<RestResp<String>, ServerError> {
    biz_ok!(rest_config.name.to_string())
}

/// Add a new pet to the store(json)
#[post(path = "/store/pet/json")]
pub async fn add_json_pet(
    State(_rest_config): State<RestConfig>,
    Json(pet): Json<Pet>,
) -> Result<RestResp<Pet>, ServerError> {
    biz_ok!(pet)
}

/// Add a new pet to the store(form)
#[post(path = "/store/pet/form")]
pub async fn add_form_pet(
    State(_rest_config): State<RestConfig>,
    Form(_pet): Form<PetForm>,
) -> Result<RestResp<Pet>, ServerError> {
    biz_err!("failed".to_string())
}

/// Get pet by id
#[get(path = "/store/pet/{id}")]
pub async fn get_pet_name(Path(params): Path<Params>) -> Result<RestResp<Pet>, ServerError> {
    biz_ok!(Pet {
        id: params.id,
        name: "Doggy".to_string(),
        tag: None,
        inline: None,
        meta: Meta {
            name: "Doggy".to_string(),
            age: 1
        },
    })
}

/// Get pet list
#[post(path = "/store/pet/list/{page}/{count}")]
pub async fn pet_page_list(
    State(_rest_config): State<RestConfig>,
    Path(_page): Path<Page>,
) -> Result<RestResp<Vec<Pet>>, ServerError> {
    biz_ok!(vec![])
}

/// Get Pet list by id
#[get(path = "/store/pet/list/{page}/{count}/{id}")]
pub async fn get_pet_name_list(
    Path(_page): Path<Page>,
    Path(params): Path<Params>,
) -> Result<RestResp<Pet>, ServerError> {
    biz_ok!(Pet {
        id: params.id,
        name: "Doggy".to_string(),
        tag: None,
        inline: None,
        meta: Meta {
            name: "Doggy".to_string(),
            age: 1
        },
    })
}

/// Query pet by id
#[get(path = "/store/pet")]
pub async fn get_query_pet_name(
    Query(query): Query<QueryPet>,
) -> Result<RestResp<Pet>, ServerError> {
    biz_ok!(Pet {
        id: query.id,
        name: "Doggy".to_string(),
        tag: None,
        inline: None,
        meta: Meta {
            name: "Doggy".to_string(),
            age: 1
        },
    })
}
