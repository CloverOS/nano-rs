use axum::extract::State;
use example_modular_model::{Meta, Page, PageData, Pet, ServiceContext};
use nano_rs::axum::errors::ServerError;
use nano_rs::axum::extractor::Path;
use nano_rs::axum::rest::RestResp;
use nano_rs::{biz_ok, post};

/// Get pet list with page data wrapper
#[post(path = "/store/pet/page-data/{page}/{count}")]
pub async fn handler(
    State(_svc): State<ServiceContext>,
    Path(page): Path<Page>,
) -> Result<RestResp<PageData<Pet>>, ServerError> {
    let pet = Pet {
        id: 1,
        name: "Doggy".to_string(),
        tag: Some("demo".to_string()),
        meta: Meta { age: 1 },
    };
    biz_ok!(PageData {
        total: 1,
        page: page.page,
        count: page.count,
        data: vec![pet],
    })
}
