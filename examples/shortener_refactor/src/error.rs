use axum::response::{Html, IntoResponse};
use http::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShortenError {
    #[error("{0}")]
    EnvError(#[from] dotenvy::Error),
    #[error("{0}")]
    SqlError(#[from] sqlx::Error),
    #[error("id: {0} not found!")]
    Notfound(String),
}

impl IntoResponse for ShortenError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(format!("<h1>{}</h1>", self)),
        )
            .into_response()
    }
}
