use axum::{Form, Json};
use axum::extract::{Query, State};

use nano_rs::{biz_err, get, post};
use nano_rs::axum::errors::ServerError;
use nano_rs::axum::extractor::path::Path;
use nano_rs::axum::rest::{biz_err, biz_ok, RestResp};
use nano_rs::config::rest::RestConfig;

use crate::model::pet::{Meta, Page, Params, Pet, PetForm, QueryPet};

/// Get the default pet store name
#[utoipa::path(
    get,
    path = "/store/name",
    tag = "Store",
    responses(
        (status = 200, body = String)
    )
)]
#[get(path = "/store/name", group = "Store", layers = ["crate::layers::auth::auth_token1"])]
pub async fn get_store_name() -> Result<RestResp<String>, ServerError> {
    biz_ok("Doggy Store".to_string())
}

/// Get Store's telephone number
#[utoipa::path(
    get,
    path = "/store/tel",
    tag = "Store",
    responses(
        (status = 200, body = String)
    )
)]
#[get(
    path = "/store/tel", group = "Store", layers = ["crate::layers::auth::auth_token#{crate::ServiceContext}"]
)]
pub async fn get_store_tel(State(rest_config): State<RestConfig>) -> Result<RestResp<String>, ServerError> {
    biz_ok(rest_config.name.to_string())
}

/// Add a new pet to the store(json)
#[utoipa::path(
    post,
    path = "/store/pet/json",
    tag = "Store",
    request_body = Pet,
    responses(
    (status = 200, body = Pet)
    )
)]
#[post(path = "/store/pet/json", group = "Store")]
pub async fn add_json_pet(State(_rest_config): State<RestConfig>, Json(pet): Json<Pet>) -> Result<RestResp<Pet>, ServerError> {
    biz_ok(pet)
}

/// Add a new pet to the store(form)
#[utoipa::path(
    post,
    path = "/store/pet/form",
    tag = "Store",
    request_body(content = PetForm, content_type = "application/x-www-form-urlencoded"),
    responses(
        (status = 200, body = Pet)
    )
)]
#[post(path = "/store/pet/form", group = "Store")]
pub async fn add_form_pet(State(_rest_config): State<RestConfig>, Form(_pet): Form<PetForm>) -> Result<RestResp<Pet>, ServerError> {
    biz_err!("failed".to_string())
}

/// Get pet by id
#[utoipa::path(
    get,
    path = "/store/pet/{id}",
    tag = "Store",
    params(Params),
    responses(
        (status = 200, body = Pet)
    )
)]
#[get(path = "/store/pet/:id", group = "Store")]
pub async fn get_pet_name(Path(params): Path<Params>) -> Result<RestResp<Pet>, ServerError> {
    biz_ok(Pet {
        id: params.id,
        name: "Doggy".to_string(),
        tag: None,
        inline: None,
        meta: Meta { name: "Doggy".to_string(), age: 1 },
    })
}


/// Get pet list
#[utoipa::path(
    post,
    path = "/store/pet/list/{page}/{count}",
    tag = "Store",
    params(Page),
    responses(
        (status = 200, body = [Pet])
    )
)]
#[post(path = "/store/pet/list/:page/:count", group = "Store")]
pub async fn pet_page_list(State(_rest_config): State<RestConfig>, Path(_page): Path<Page>) -> Result<RestResp<Vec<Pet>>,
    ServerError> {
    biz_ok(vec![])
}

/// Get pet list by id
///
/// Get Pet list by id
#[utoipa::path(
    get,
    path = "/store/pet/list/{page}/{count}/{id}",
    tag = "Store",
    params(
        Page,
        Params,
    ),
    responses(
        (status = 200, body = Pet)
    )
)]
#[get(path = "/store/pet/list/:page/:count/:id", group = "Store")]
pub async fn get_pet_name_list(Path(_page): Path<Page>, Path(params): Path<Params>) -> Result<RestResp<Pet>, ServerError> {
    biz_ok(Pet {
        id: params.id,
        name: "Doggy".to_string(),
        tag: None,
        inline: None,
        meta: Meta { name: "Doggy".to_string(), age: 1 },
    })
}

/// Query pet by id
#[utoipa::path(
    get,
    path = "/store/pet",
    tag = "Store",
    params(QueryPet),
    responses(
        (status = 200, body = Pet)
    )
)]
#[get(path = "/store/pet", group = "Store")]
pub async fn get_query_pet_name(Query(query): Query<QueryPet>) -> Result<RestResp<Pet>, ServerError> {
    biz_ok(Pet {
        id: query.id,
        name: "Doggy".to_string(),
        tag: None,
        inline: None,
        meta: Meta { name: "Doggy".to_string(), age: 1 },
    })
}
