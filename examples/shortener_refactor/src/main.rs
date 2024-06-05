use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use shortener_refactor::{not_found, redirect_handler, shortener_handler, AppState, LISTEN_ADDR};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    // init tracing
    let layer = fmt::layer().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

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
