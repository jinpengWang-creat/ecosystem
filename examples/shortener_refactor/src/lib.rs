mod error;
mod state;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
pub use error::ShortenError;
use http::{header::LOCATION, HeaderMap, StatusCode, Uri};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
pub use state::AppState;
use tracing::info;

lazy_static::lazy_static! {
    pub static ref LISTEN_ADDR: String = dotenvy::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8888".to_string());
}

#[derive(FromRow, Debug, Deserialize)]
struct ShortenUrl {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct ShortenRequest {
    url: String,
}
#[derive(Debug, Serialize)]
pub struct ShortenResponse {
    url: String,
}

pub async fn shortener_handler(
    State(state): State<AppState>,
    Json(ShortenRequest { url }): Json<ShortenRequest>,
) -> Result<impl IntoResponse, ShortenError> {
    let id = state.shorten(&url).await?;
    let body = Json(ShortenResponse {
        url: format!("http://{}/{}", &*LISTEN_ADDR, id),
    });
    Ok((StatusCode::CREATED, body))
}

pub async fn redirect_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ShortenError> {
    let url = state.get_url(&id).await?;
    info!("Redirected to: {}", url);
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url.parse().unwrap());
    Ok((StatusCode::PERMANENT_REDIRECT, headers, ""))
}

pub async fn not_found(uri: Uri) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        ShortenError::Notfound(format!("{}", uri)),
    )
}
