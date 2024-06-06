use axum::{
    extract::rejection::JsonRejection,
    response::{Html, IntoResponse},
};
use http::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShortenError {
    #[error("{0}")]
    EnvError(#[from] dotenvy::Error),
    #[error("{0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("id: {0} not found!")]
    Notfound(String),
    #[error(transparent)]
    JsonRejectionError(#[from] JsonRejection),
}

impl IntoResponse for ShortenError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ShortenError::EnvError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("<h1>{}</h1>", e)),
            ),
            ShortenError::Notfound(uri) => (
                StatusCode::NOT_FOUND,
                Html(format!("<h1>{} not found!</h1>", uri)),
            ),
            ShortenError::DatabaseError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("<h1>{}</h1>", e)),
            ),
            ShortenError::JsonRejectionError(json_rejection) => (
                json_rejection.status(),
                Html(format!("<h1>{}</h1>", json_rejection.body_text())),
            ),
        }
        .into_response()
    }
}
