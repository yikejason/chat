use std::net::SocketAddr;

use anyhow::Result;
use chat_server::{get_router, AppConfig};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = fmt::layer().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::load()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);
    let app = get_router(config).await?;

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
