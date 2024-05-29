use std::time::Duration;

use anyhow::Result;
use axum::{routing::get, Router};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::runtime;
use opentelemetry_sdk::trace;
use opentelemetry_sdk::trace::RandomIdGenerator;
use opentelemetry_sdk::trace::Tracer;
use opentelemetry_sdk::Resource;
use tokio::{net::TcpListener, time::sleep};
use tracing::{debug, info, instrument, level_filters::LevelFilter, warn};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    Layer,
};

#[tokio::main]
async fn main() -> Result<()> {
    let file_appender = tracing_appender::rolling::minutely("/tmp/logs", "ecosystem.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // console layer for tracing-subscriber
    let console = fmt::layer()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::DEBUG);

    // file appender layer for tracing-subscriber
    let file_appender = fmt::Layer::new()
        .with_writer(non_blocking)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_filter(LevelFilter::INFO);

    // opentelemetry tracing layer for tracing-subscriber
    let tracer = init_tracer()?;
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(console)
        .with(file_appender)
        .with(opentelemetry)
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
    info!(http.status_code = 200, "Request completed");
    warn!("index handler spend too long!");
    ret
}

#[instrument]
async fn long_task() -> &'static str {
    sleep(Duration::from_millis(111)).await;
    "hello world"
}

fn init_tracer() -> Result<Tracer> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            trace::config()
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(32)
                .with_max_attributes_per_span(64)
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "axum-tracing",
                )])),
        )
        .install_batch(runtime::Tokio)?;
    Ok(tracer)
}
