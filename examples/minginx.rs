use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    listening_addr: String,
    upstream_addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = fmt::layer().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = prase_config()?;
    info!("config: {:?}", config);

    let listener = TcpListener::bind(&config.listening_addr).await?;
    info!("listening on: {}", config.listening_addr);

    let config = Arc::new(config);
    loop {
        let (socket, remote_addr) = listener.accept().await?;
        info!("accepted connection from: {:?}", remote_addr);
        let cloned_config = Arc::clone(&config);
        tokio::spawn(async move {
            let upstream = TcpStream::connect(&cloned_config.upstream_addr).await?;
            proxy(upstream, socket).await?;
            Ok::<(), anyhow::Error>(())
        });
    }
}

fn prase_config() -> Result<Config> {
    Ok(Config {
        listening_addr: "0.0.0.0:8081".to_string(),
        upstream_addr: "127.0.0.1:8080".to_string(),
    })
}

async fn proxy(upstream: TcpStream, downstream: TcpStream) -> Result<()> {
    let (mut upstream_read, mut upstream_write) = upstream.into_split();
    let (mut downstream_read, mut downstream_write) = downstream.into_split();

    let up_to_down = tokio::io::copy(&mut upstream_read, &mut downstream_write);
    let down_to_up = tokio::io::copy(&mut downstream_read, &mut upstream_write);

    if let Err(e) = tokio::try_join!(up_to_down, down_to_up) {
        warn!("error: {:?}", e);
    }

    Ok(())
}
