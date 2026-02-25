/// @tag Store
pub mod store;

/// @tag Samoyed
pub mod samoyed {
    use axum::extract::{Path, State};

    use nano_rs::axum::errors::ServerError;
    use nano_rs::axum::rest::RestResp;
    use nano_rs::{biz_ok, get, post};

    use crate::ServiceContext;
    use crate::types::pet::PetShower;

    /// Give your Samoyed a bath
    #[post(path = "/samoyed/shower", layers = ["crate::layers::auth::auth_token#{crate::ServiceContext}", "crate::layers::auth::auth_token1"])]
    pub async fn shower(
        State(_svc): State<ServiceContext>,
    ) -> Result<RestResp<PetShower>, ServerError> {
        biz_ok!(PetShower {
            name: "mantou".to_string(),
            status: "clean! Ready to go home".to_string(),
        })
    }

    /// Get Samoyed name
    #[get(path = "/samoyed/name")]
    pub async fn name(State(_svc): State<ServiceContext>) -> Result<RestResp<String>, ServerError> {
        biz_ok!("mantou".to_string())
    }

    /// Say Hello to name
    #[get(path = "/samoyed/{name}")]
    pub async fn hello(
        Path(name): Path<String>,
        State(_svc): State<ServiceContext>,
    ) -> Result<RestResp<String>, ServerError> {
        biz_ok!(name)
    }

    /// Miss mantou so much
    #[get(path = "/samoyed/miss")]
    pub async fn miss() -> Result<RestResp<String>, ServerError> {
        let _ = std::fs::read("pass")?;
        biz_ok!("pass".to_string())
    }
}
