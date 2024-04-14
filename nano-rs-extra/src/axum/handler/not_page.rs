use axum::response::{Html, IntoResponse};

pub async fn handler_404() -> impl IntoResponse {
    Html("<h5>404 page</h5>")
}