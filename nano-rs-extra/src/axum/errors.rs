use axum::response::{IntoResponse, Response};
use crate::axum::rest::RestResp;

pub struct ServerError(anyhow::Error);

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        tracing::error!("INTERNAL_SERVER_ERROR: {} - {:#?}", self.0, self.0.backtrace());
        RestResp {
            code: 500,
            msg: "INTERNAL_SERVER_ERROR".to_string(),
            data: (),
        }
            .into_response()
    }
}

impl<E> From<E> for ServerError
    where
        E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
