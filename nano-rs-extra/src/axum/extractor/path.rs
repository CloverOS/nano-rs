use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::extract::path::ErrorKind;
use axum::extract::rejection::PathRejection;
use axum::http::request::Parts;
use axum::http::StatusCode;
use serde::de::DeserializeOwned;
use crate::axum::rest::RestResp;

// We define our own `Path` extractor that customizes the error from `axum::extract::Path`
pub struct Path<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for Path<T>
    where
    // these trait bounds are copied from `impl FromRequest for axum::extract::path::Path`
        T: DeserializeOwned + Send,
        S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<RestResp<()>>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let (status, body) = match rejection {
                    PathRejection::FailedToDeserializePathParams(inner) => {
                        let mut status = StatusCode::BAD_REQUEST;

                        let kind = inner.into_kind();
                        let body = match &kind {
                            ErrorKind::WrongNumberOfParameters { .. } => RestResp {
                                code: 500,
                                msg: kind.to_string(),
                                data: (),
                            },

                            ErrorKind::ParseErrorAtKey { .. } => RestResp {
                                code: 500,
                                msg: kind.to_string(),
                                data: (),
                            },

                            ErrorKind::ParseErrorAtIndex { .. } => RestResp {
                                code: 500,
                                msg: kind.to_string(),
                                data: (),
                            },

                            ErrorKind::ParseError { .. } => RestResp {
                                code: 500,
                                msg: kind.to_string(),
                                data: (),
                            },

                            ErrorKind::InvalidUtf8InPathParam { key: _key } => RestResp {
                                code: 500,
                                msg: kind.to_string(),
                                data: (),
                            },

                            ErrorKind::UnsupportedType { .. } => {
                                // this error is caused by the programmer using an unsupported type
                                // (such as nested maps) so respond with `500` instead
                                status = StatusCode::INTERNAL_SERVER_ERROR;
                                RestResp {
                                    code: 500,
                                    msg: kind.to_string(),
                                    data: (),
                                }
                            }

                            ErrorKind::Message(msg) => RestResp {
                                code: 500,
                                msg: msg.clone(),
                                data: (),
                            },

                            _ => RestResp {
                                code: 500,
                                msg: format!("Unhandled deserialization error: {kind}"),
                                data: (),
                            },
                        };

                        (status, body)
                    }
                    PathRejection::MissingPathParams(error) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        RestResp {
                            code: 500,
                            msg: error.to_string(),
                            data: (),
                        },
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        RestResp {
                            code: 500,
                            msg: format!("Unhandled path rejection: {rejection}"),
                            data: (),
                        },
                    ),
                };

                Err((status, axum::Json(body)))
            }
        }
    }
}
