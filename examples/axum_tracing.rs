use std::time::Duration;

use anyhow::Result;
use axum::{routing::get, Router};
use tokio::{net::TcpListener, time::sleep};
use tracing::{debug, info, instrument, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

#[tokio::main]
async fn main() -> Result<()> {
    let file_appender = tracing_appender::rolling::minutely("/tmp/logs", "ecosystem.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let console = fmt::Layer::new()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::DEBUG);

    let file_console = fmt::Layer::new()
        .with_writer(non_blocking)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(console)
        .with(file_console)
        .init();

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/long", get(long_task));

    let addr = "0.0.0.0:8080";

    let lisitener = TcpListener::bind(addr).await?;
    info!("Starting server on {}", addr);
    axum::serve(lisitener, app.into_make_service()).await?;
    Ok(())
}

#[instrument]
async fn index_handler() -> &'static str {
    debug!("index handler start!");
    sleep(Duration::from_millis(11)).await;
    let ret = long_task().await;
    info!(http.status = 200, "Request completed");
    warn!("index handler spend too long!");
    ret
}

#[instrument]
async fn long_task() -> &'static str {
    sleep(Duration::from_millis(111)).await;
    "hello world"
}
