use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use tracing::info;

use crate::ShortenError;
const MAX_CONNECTION: u32 = 12;
#[derive(Debug, Clone)]
pub struct AppState {
    db: PgPool,
}

#[derive(FromRow, Debug, Deserialize)]
struct ShortenUrl {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
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
