use std::net::SocketAddr;

use anyhow::Result;
use notify_server::get_router;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = fmt::layer().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 6687));

    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);
    let app = get_router();

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
