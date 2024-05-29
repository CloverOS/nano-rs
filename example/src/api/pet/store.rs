use axum::{Form, Json};
use axum::extract::{Query, State};

use nano_rs::{get, post};
use nano_rs::axum::errors::ServerError;
use nano_rs::axum::extractor::path::Path;
use nano_rs::axum::rest::{biz_ok, RestResp};
use nano_rs::config::rest::RestConfig;

use crate::model::pet::{Meta, Page, Params, Pet, QueryPet};

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

/// Add a new pet to the store
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

/// Add a new pet to the store
#[utoipa::path(
        post,
        path = "/store/pet/list/{page}/{count}",
        tag = "Store",
        request_body = Pet,
        responses(
            (status = 200, body = Pet)
        )
    )]
#[post(path = "/store/pet/list/:page/:count", group = "Store")]
pub async fn pet_page_list(State(_rest_config): State<RestConfig>, Path(_page): Path<Page>, Json(pet): Json<Pet>) -> Result<RestResp<Pet>,
    ServerError> {
    biz_ok(pet)
}

/// Add a new pet to the store
#[utoipa::path(
        post,
        path = "/store/pet/form",
        tag = "Store",
        params(Pet),
        responses(
            (status = 200, body = Pet)
        )
    )]
#[post(path = "/store/pet/form", group = "Store")]
pub async fn add_form_pet(State(_rest_config): State<RestConfig>, Form(pet): Form<Pet>) -> Result<RestResp<Pet>, ServerError> {
    biz_ok(pet)
}

/// Get pet by id
#[utoipa::path(
    get,
    path = "/store/pet/{id}",
    tag = "Store",
    params(
         ("id", description = "id"),
    ),
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
///
/// Get Pet list by id
#[utoipa::path(
    get,
    path = "/store/pet/list/{page}/{count}/{id}",
    tag = "Store",
    params(
        ("page", description = "page"),
        ("count", description = "count"),
        ("id", description = "id")
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

/// Get pet by id
#[utoipa::path(
    get,
    path = "/store/pet",
    tag = "Store",
    params(QueryPet),
    responses(
        (status = 200, body = Pet)
    )
)]
#[get(path = "/store/pet",group = "Store")]
pub async fn get_query_pet_name(Query(query): Query<QueryPet>) -> Result<RestResp<Pet>, ServerError> {
    biz_ok(Pet {
        id: query.id,
        name: "Doggy".to_string(),
        tag: None,
        inline: None,
        meta: Meta { name: "Doggy".to_string(), age: 1 },
    })
}
