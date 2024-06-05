use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tracing::info;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};

use http::{header::LOCATION, HeaderMap, StatusCode, Uri};
use serde::{Deserialize, Serialize};

use crate::{AppState, ShortenError, LISTEN_ADDR};

pub async fn run() -> Result<()> {
    // init app state
    let state = AppState::try_new().await?;

    // bind listener
    let listener = TcpListener::bind(&*LISTEN_ADDR).await?;
    info!("Listening on: {}", &*LISTEN_ADDR);
    // init app router
    let app = Router::new()
        .route("/", post(shortener_handler))
        .route("/:id", get(redirect_handler))
        .fallback(not_found)
        .with_state(state);

    // init server
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ShortenRequest {
    url: String,
}
#[derive(Debug, Serialize)]
pub struct ShortenResponse {
    url: String,
}

async fn shortener_handler(
    State(state): State<AppState>,
    Json(ShortenRequest { url }): Json<ShortenRequest>,
) -> Result<impl IntoResponse, ShortenError> {
    let id = state.shorten(&url).await?;
    let body = Json(ShortenResponse {
        url: format!("http://{}/{}", &*LISTEN_ADDR, id),
    });
    Ok((StatusCode::CREATED, body))
}

async fn redirect_handler(
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
