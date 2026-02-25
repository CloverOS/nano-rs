use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use example_modular_model::ServiceContext;
use nano_rs::axum::rest::RestResp;

pub async fn auth_token(
    State(_svc): State<ServiceContext>,
    request: Request,
    next: Next,
) -> Response {
    next.run(request).await
}

pub async fn auth_token1(request: Request, next: Next) -> Result<Response, RestResp<()>> {
    if false {
        Ok(next.run(request).await)
    } else {
        Err(RestResp::<()> {
            code: 502,
            msg: "auth failed".to_string(),
            data: None,
        })
    }
}
