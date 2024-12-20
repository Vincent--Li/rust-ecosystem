use std::sync::Arc;

use anyhow::Result;
use derive_more::derive::Debug;
use serde::{Deserialize, Serialize};
use tokio::{
    io,
    net::{TcpListener, TcpStream},
};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    listen_addr: String,
    upstream_addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = resolve_config();
    let config = Arc::new(config);
    info!("Upstream address: {}", config.upstream_addr);
    info!("Listen address: {}", config.listen_addr);

    let listener = TcpListener::bind(&config.listen_addr).await?;
    loop {
        let (client, addr) = listener.accept().await?;
        let cloned_config = Arc::clone(&config);
        info!("New connection from {}", addr);
        tokio::spawn(async move {
            let upstream = TcpStream::connect(&cloned_config.upstream_addr).await?;
            proxy(client, upstream).await?;
            Ok::<(), anyhow::Error>(())
        });
    }

    #[allow(unreachable_code)]
    Ok(())
}

async fn proxy(mut client: TcpStream, mut upstream: TcpStream) -> Result<()> {
    let (mut client_reader, mut client_writer) = client.split();
    let (mut upstream_reader, mut upstream_writer) = upstream.split();

    let client_to_upstream = io::copy(&mut client_reader, &mut upstream_writer);

    let upstream_to_client = io::copy(&mut upstream_reader, &mut client_writer);

    if let Err(e) = tokio::try_join!(client_to_upstream, upstream_to_client) {
        warn!("Error: {}", e);
    };
    Ok(())
}

fn resolve_config() -> Config {
    Config {
        listen_addr: "127.0.0.1:3000".to_string(),
        upstream_addr: "127.0.0.1:3001".to_string(),
    }
}
