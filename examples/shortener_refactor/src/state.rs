use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use tracing::{info, warn};

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

    // id error message: Err(Database(PgDatabaseError { severity: Error, code: "23505", message: "重复键违反唯一约束\"shorten_urls_pkey\"", detail: Some("键值\"(id)=(1     )\" 已经存在"), hint: None, position: None, where: None, schema: Some("public"), table: Some("shorten_urls"), column: None, data_type: None, constraint: Some("shorten_urls_pkey"), file: Some("nbtinsert.c"), line: Some(673), routine: Some("_bt_check_unique") }))
    pub async fn shorten(&self, url: &str) -> Result<String, ShortenError> {
        info!("short url: {:?}", url);
        loop {
            match self.insert_url(url).await {
                Ok(id) => break Ok(id),
                Err(ShortenError::DatabaseError(sqlx::Error::Database(err)))
                    if Some("23505".into()).eq(&err.code()) =>
                {
                    warn!("id conflict! error: {:?}", err);
                }
                Err(e) => break Err(e),
            };
        }
    }

    async fn insert_url(&self, url: &str) -> Result<String, ShortenError> {
        info!("url: {} not found, do insert", url);
        let id = nanoid::nanoid!(6);
        let row: ShortenUrl =
            sqlx::query_as("INSERT INTO shorten_urls (id, url) VALUES ($1, $2) ON CONFLICT(url) DO UPDATE SET url = EXCLUDED.url RETURNING *")
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
