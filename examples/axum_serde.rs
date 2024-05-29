use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::State,
    routing::{get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{info, instrument, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

#[derive(Debug, PartialEq, derive_builder::Builder, Clone, Serialize, Deserialize)]
#[builder(setter(into))]
struct User {
    name: String,
    age: u8,
    skills: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct UserUpdate {
    age: Option<u8>,
    skills: Option<Vec<String>>,
}
#[tokio::main]
async fn main() -> Result<()> {
    let console = tracing_subscriber::fmt::layer()
        .with_span_events(FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(console).init();
    let addr = "0.0.0.0:8080";

    let user = User {
        name: "Alice".to_string(),
        age: 30,
        skills: vec!["Rust".to_string(), "TypeScript".to_string()],
    };
    let user = Arc::new(Mutex::new(user));
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/user", get(user_handler))
        .route("/update", patch(update_handler))
        .with_state(user);

    info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[instrument]
async fn index_handler() -> &'static str {
    "Hello, World!"
}

#[instrument]
async fn user_handler(State(user): State<Arc<Mutex<User>>>) -> Json<User> {
    let lock = user.lock().await;
    (*lock).clone().into()
}

#[instrument]
async fn update_handler(
    State(user): State<Arc<Mutex<User>>>,
    Json(UserUpdate { age, skills }): Json<UserUpdate>,
) -> Json<User> {
    let mut lock = user.lock().await;
    if let Some(age) = age {
        lock.age = age;
    }
    if let Some(skills) = skills {
        lock.skills = skills;
    }
    (*lock).clone().into()
}
