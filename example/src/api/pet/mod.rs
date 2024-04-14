pub mod store;

pub mod samoyed {
    use axum::extract::{Path, State};

    use nano_rs::axum::errors::ServerError;
    use nano_rs::axum::rest::{biz_err, biz_ok, RestResp};
    use nano_rs::{get, post};

    use crate::ServiceContext;
    use crate::types::pet::PetShower;

    #[post(path = "/samoyed/shower", group = "Samoyed", api = "Give your Samoyed a bath", layers =
    ["crate::layers::auth::auth_token#{crate::ServiceContext}", "crate::layers::auth::auth_token1"])]
    pub async fn shower(State(_svc): State<ServiceContext>) -> Result<RestResp<PetShower>, ServerError> {
        biz_ok(PetShower {
            name: "mantou".to_string(),
            status: "clean! Ready to go home".to_string(),
        })
    }

    #[get(path = "/samoyed/name", group = "Samoyed", api = "Get Samoyed name")]
    pub async fn name(State(_svc): State<ServiceContext>) -> Result<RestResp<String>, ServerError> {
        biz_ok("mantou".to_string())
    }

    #[get(path = "/samoyed/:name", group = "Samoyed", api = "Say Hello to name")]
    pub async fn hello(Path(name): Path<String>, State(_svc): State<ServiceContext>) -> Result<RestResp<String>, ServerError> {
        biz_ok(name)
    }

    #[get(path = "/samoyed/miss", group = "Samoyed", api = "Miss mantou so much")]
    pub async fn miss() -> Result<RestResp<()>, ServerError> {
        biz_err(520, "Already in heaven".to_string())
    }
}
