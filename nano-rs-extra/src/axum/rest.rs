use std::fmt::Display;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use bytes::{BufMut, BytesMut};
use serde::{Deserialize, Serialize};

use crate::axum::errors::ServerError;

#[derive(Serialize, Deserialize, Clone)]
pub struct RestResp<T> {
    pub code: i32,
    pub msg: String,
    pub data: Option<T>,
}

impl<T> IntoResponse for RestResp<T> where T: Serialize {
    fn into_response(self) -> Response {
        // Use a small initial capacity of 128 bytes like serde_json::to_vec
        // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
        let mut buf = BytesMut::with_capacity(128).writer();
        match serde_json::to_writer(&mut buf, &self) {
            Ok(()) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )],
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                err.to_string(),
            )
                .into_response(),
        }
    }
}


///通用的返回错误的方法
/// Common return error method
/// # Example
/// ```rust
///
/// use nano_rs_extra::axum::rest::biz_err;
/// use nano_rs_extra::axum::errors::ServerError;
/// use axum::Json;
/// use nano_rs_extra::axum::rest::RestResp;
///
/// pub async fn test() -> Result<RestResp<()>, ServerError> {
///    biz_err(500, "error".to_string())
/// }
/// ```
pub fn biz_err<T, S>(code: i32, msg: S) -> Result<RestResp<T>, ServerError> where S: ToString + Display + std::fmt::Debug {
    tracing::error!("code: {} - {:#?}",code,msg);
    Ok(RestResp {
        code,
        msg: msg.to_string(),
        data: None,
    })
}

///通用的返回成功的方法
/// Common return success method
/// # Example
///```rust
/// use nano_rs_extra::axum::rest::biz_ok;
/// use nano_rs_extra::axum::rest::RestResp;
/// use axum::Json;
/// use axum::response::IntoResponse;
/// use nano_rs_extra::axum::errors::ServerError;
///
/// pub struct TestStruct {
///    pub name: String,
/// }
///
/// pub async fn test() -> Result<RestResp<TestStruct>, ServerError> {
///   biz_ok(200,TestStruct{
///     name: "test".to_string(),
///   })
/// }
/// ```
pub fn biz_ok<T>(code: i32, data: T) -> Result<RestResp<T>, ServerError> {
    Ok(RestResp {
        code,
        msg: "Success".to_string(),
        data: Some(data),
    })
}


/// biz_ok macro
///
/// # Example
/// ```rust
/// fn main() {
///     use nano_rs_extra::axum::errors::ServerError;
///     use nano_rs_extra::axum::rest::RestResp;
///     use nano_rs_extra::biz_ok;
///     let result: Result<RestResp<()>, ServerError> = biz_ok!();
///     let result_with_msg: Result<RestResp<()>, ServerError> = biz_ok!("Custom message");
///     let custom_result: Result<RestResp<()>, ServerError> = biz_ok!(203, "Yes");
/// }
/// ```
///
#[macro_export]
macro_rules! biz_ok {
    ($data:expr) => {
        nano_rs::axum::rest::biz_ok(200, $data)
    };

    ($code:expr, $data:expr) => {{
        nano_rs::axum::rest::biz_ok($code, $data)
    }};

    () => {
        nano_rs::axum::rest::biz_ok(200, "Success".to_string())
    };
}


/// biz_err macro
///
/// # Example
/// ```rust
/// fn main() {
///     use nano_rs_extra::axum::errors::ServerError;
///     use nano_rs_extra::axum::rest::RestResp;
///     use nano_rs_extra::biz_err;
///     let result: Result<RestResp<()>, ServerError> = biz_err!();
///     let result_with_msg: Result<RestResp<()>, ServerError> = biz_err!("Custom message");
///     let custom_result: Result<RestResp<()>, ServerError> = biz_err!(404, "Not Found");
/// }
/// ```
///
#[macro_export]
macro_rules! biz_err {
    ($msg:expr) => {
        nano_rs::axum::rest::biz_err(500, $msg)
    };

    ($code:expr, $msg:expr) => {{
        nano_rs::axum::rest::biz_err($code, $msg)
    }};

    () => {
        nano_rs::axum::rest::biz_err(500, "error".to_string())
    };
}