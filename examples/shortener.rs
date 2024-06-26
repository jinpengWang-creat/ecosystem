use anyhow::Result;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_macros::debug_handler;
use http::{header::LOCATION, HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[derive(Debug, Deserialize)]
struct ShortenRequest {
    url: String,
}

#[derive(Debug, Serialize)]
struct ShortenResponse {
    url: String,
}

#[derive(Debug, Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Debug, FromRow)]
struct UrlRecord {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}

const LISTEN_ADDR: &str = "0.0.0.0:9876";
#[tokio::main]
async fn main() -> Result<()> {
    let layer = fmt::layer().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    info!("Listening on: {}", LISTEN_ADDR);

    let url = "postgres://fandream:fandream@localhost/shortener";
    let state = AppState::try_new(url).await?;
    info!("Connection to the database: {}", url);

    let app = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state(state);

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn shorten(
    State(state): State<AppState>,
    Json(data): Json<ShortenRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let id = state
        .shorten(&data.url)
        .await
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    let body = Json(ShortenResponse {
        url: format!("http://{}/{}", LISTEN_ADDR, id),
    });
    Ok((StatusCode::CREATED, body))
}

#[debug_handler]
async fn redirect(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Redirecting to: {}", id);
    let url = state
        .get_url(&id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    info!("Redirected to: {}", url);
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url.parse().unwrap());
    Ok((StatusCode::PERMANENT_REDIRECT, headers, ""))
}

impl AppState {
    async fn try_new(url: &str) -> Result<Self> {
        let db = PgPool::connect(url).await?;
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS urls (
                id CHAR(6) PRIMARY KEY,
                url TEXT NOT NULL UNIQUE
            )"#,
        )
        .execute(&db)
        .await?;
        Ok(Self { db })
    }

    async fn shorten(&self, url: &str) -> Result<String> {
        let id = nanoid::nanoid!(6);
        let record: UrlRecord = sqlx::query_as(
            "INSERT INTO urls (id, url) VALUES ($1, $2) ON CONFLICT(url) DO UPDATE SET url = EXCLUDED.url RETURNING *",
        )
        .bind(&id)
        .bind(url)
        .fetch_one(&self.db)
        .await?;
        info!("Shortened: {} -> {}", record.id, record.url);
        Ok(record.id)
    }

    async fn get_url(&self, id: &str) -> Result<String> {
        let record: UrlRecord = sqlx::query_as("SELECT id, url FROM urls WHERE id = $1")
            .bind(id)
            .fetch_one(&self.db)
            .await?;
        info!("Redirecting: {} -> {}", id, record.url);
        Ok(record.url)
    }
}
