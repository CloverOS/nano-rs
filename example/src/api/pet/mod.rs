pub mod store;

pub mod samoyed {
    use axum::extract::{Path, State};

    use nano_rs::{biz_ok, get, post};
    use nano_rs::axum::errors::ServerError;
    use nano_rs::axum::rest::RestResp;

    use crate::ServiceContext;
    use crate::types::pet::PetShower;

    /// Give your Samoyed a bath
    #[utoipa::path(
        post,
        path = "/samoyed/shower",
        tag = "Samoyed",
        responses(
            (status = 200, body = PetShower)
        )
    )]
    #[post(layers = ["crate::layers::auth::auth_token#{crate::ServiceContext}", "crate::layers::auth::auth_token1"])]
    pub async fn shower(State(_svc): State<ServiceContext>) -> Result<RestResp<PetShower>, ServerError> {
        biz_ok!(PetShower {
            name: "mantou".to_string(),
            status: "clean! Ready to go home".to_string(),
        })
    }

    /// Get Samoyed name
    #[utoipa::path(
        get,
        path = "/samoyed/name",
        tag = "Samoyed",
        responses(
            (status = 200, body = String)
        )
    )]
    #[get()]
    pub async fn name(State(_svc): State<ServiceContext>) -> Result<RestResp<String>, ServerError> {
        biz_ok!("mantou".to_string())
    }

    /// Say Hello to name
    #[utoipa::path(
        get,
        path = "/samoyed/{name}",
        tag = "Samoyed",
        params(
            ("name", description = "pet name"),
        ),
        responses(
            (status = 200, body = String)
        )
    )]
    #[get()]
    pub async fn hello(Path(name): Path<String>, State(_svc): State<ServiceContext>) -> Result<RestResp<String>, ServerError> {
        biz_ok!(name)
    }

    /// Miss mantou so much
    #[utoipa::path(
        get,
        path = "/samoyed/miss",
        tag = "Samoyed",
        responses(
            (status = 200)
        )
    )]
    #[get()]
    pub async fn miss() -> Result<RestResp<String>, ServerError> {
        let _ = std::fs::read("pass")?;
        biz_ok!("pass".to_string())
    }
}
