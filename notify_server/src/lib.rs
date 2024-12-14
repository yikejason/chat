mod sse;

use anyhow::Result;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures::StreamExt;
use sqlx::postgres::PgListener;
use sse::*;
use tracing::info;

const INDEX_HTML: &str = include_str!("../index.html");

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse_handler))
}

// search sqlx pglistener docs to write it
pub async fn set_up_pg_listener() -> Result<()> {
    let mut listener =
        PgListener::connect("postgresql://postgres:postgres@localhost:5432/chat").await?;
    listener.listen("chat_updated").await?;
    listener.listen("chat_message_created").await?;

    let mut stream = listener.into_stream();

    tokio::spawn(async move {
        while let Some(Ok(notifi)) = stream.next().await {
            info!("Received notification: {:?}", notifi);
        }
    });

    Ok(())
}

pub(crate) async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}
