use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use nano_rs::axum::rest::RestResp;

use crate::ServiceContext;

pub async fn auth_token(State(_svc): State<ServiceContext>,request: Request, next: Next) -> Result<Response, impl IntoResponse> {
    //todo token verify things...
    if true {
        Ok(next.run(request).await)
    }else{
        return Err(RestResp::<()> {
            code: 502,
            msg: "auth server not found, please try again later".to_string(),
            data: None,
        });
    }
}

pub async fn auth_token1(request: Request, next: Next) -> Result<Response, impl IntoResponse> {
    //todo token verify things...
    if false {
        Ok(next.run(request).await)
    }else{
        return Err(RestResp::<()> {
            code: 502,
            msg: "auth server not found, please try again later".to_string(),
            data: None,
        });
    }
}