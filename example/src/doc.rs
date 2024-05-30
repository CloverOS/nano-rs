/// Code generated by nano-rs. DO NOT EDIT.
use utoipa::OpenApi;
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Pet",
        description = "Pet Api Server",
        version = "v1",
        license(name = "", url = ""),
        contact(name = "Pet", email = "pet@gmail.com", url = ""),
    ),
    paths(
        crate::api::pet::samoyed::hello,
        crate::api::pet::samoyed::miss,
        crate::api::pet::samoyed::name,
        crate::api::pet::samoyed::shower,
        crate::api::pet::store::add_form_pet,
        crate::api::pet::store::add_json_pet,
        crate::api::pet::store::get_pet_name,
        crate::api::pet::store::get_pet_name_list,
        crate::api::pet::store::get_query_pet_name,
        crate::api::pet::store::get_store_name,
        crate::api::pet::store::get_store_tel,
        crate::api::pet::store::pet_page_list
    ),
    components(
        schemas(
            crate::model::pet::InlineThings,
            crate::model::pet::Meta,
            crate::model::pet::Pet,
            crate::model::pet::PetForm,
            crate::types::pet::PetShower
        )
    ),
    servers(
        (url = "", description = "dev"),
        (url = "https://example.com", description = "prod"),
    ),
    tags()
)]
pub(super) struct GenApi {}
