use anyhow::Result;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    // init tracing
    let layer = fmt::layer().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    shortener_refactor::run().await
}
