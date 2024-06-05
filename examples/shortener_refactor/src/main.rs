use anyhow::Result;

use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use http::header::LOCATION;
use http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::{postgres::PgPoolOptions, PgPool};
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};
const MAX_CONNECTION: u32 = 12;
#[derive(Debug, Clone)]
struct AppState {
    db: PgPool,
}

#[derive(FromRow, Debug, Deserialize)]
struct ShortenUrl {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}

#[derive(Debug, Deserialize)]
struct ShortenRequest {
    url: String,
}
#[derive(Debug, Serialize)]
struct ShortenResponse {
    url: String,
}

#[derive(Debug, Error)]
enum ShortenError {
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

lazy_static::lazy_static! {
    static ref LISTEN_ADDR: String = dotenvy::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8888".to_string());
}

#[tokio::main]
async fn main() -> Result<()> {
    // init tracing
    let layer = fmt::layer().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    // init app state
    let state = AppState::try_new().await?;

    // bind listener
    let listener = TcpListener::bind(&*LISTEN_ADDR).await?;

    // init app router
    let app = Router::new()
        .route("/", post(shortener_handler))
        .route("/:id", get(redirect_handler))
        .with_state(state);

    // init server
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
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
impl AppState {
    pub async fn try_new() -> Result<Self, ShortenError> {
        let url = &dotenvy::var("DATABASE_URL")?;
        let db = PgPoolOptions::new()
            .max_connections(MAX_CONNECTION)
            .connect(url)
            .await?;
        Ok(Self { db })
    }

    pub async fn shorten(&self, url: &str) -> Result<String, ShortenError> {
        info!("short url: {:?}", url);
        let row = sqlx::query_as("SELECT id, url FROM shorten_urls WHERE url = $1")
            .bind(url)
            .fetch_optional(&self.db)
            .await?;
        info!("get row: {:?}", row);
        let id = match row {
            Some(ShortenUrl { id, .. }) => id,
            None => self.insert_url(url).await?,
        };
        Ok(id)
    }

    async fn insert_url(&self, url: &str) -> Result<String, ShortenError> {
        info!("url: {} not found, do insert", url);
        let id = loop {
            let id = nanoid::nanoid!(6);
            match sqlx::query_as("SELECT COUNT(1) FROM shorten_urls WHERE id = $1")
                .bind(&id)
                .fetch_optional(&self.db)
                .await?
            {
                Some::<(i32,)>((count,)) if count > 0 => continue,
                _ => break id,
            }
        };
        info!("get id: {:?}", id);
        let row: ShortenUrl =
            sqlx::query_as("INSERT INTO shorten_urls (id, url) VALUES ($1, $2) RETURNING id")
                .bind(id)
                .bind(url)
                .fetch_one(&self.db)
                .await?;
        Ok(row.id)
    }

    pub async fn get_url(&self, id: &str) -> Result<String, ShortenError> {
        let row: Option<ShortenUrl> =
            sqlx::query_as("SELECT id, url FROM shorten_urls WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.db)
                .await?;
        match row {
            Some(row) => Ok(row.url),
            None => Err(ShortenError::Notfound(id.to_string())),
        }
    }
}
