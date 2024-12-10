mod auth;
mod chat;
mod message;
mod workspace;

pub(crate) use auth::*;
pub(crate) use chat::*;
pub(crate) use message::*;
pub(crate) use workspace::*;

use axum::response::IntoResponse;

pub(crate) async fn index_handler() -> impl IntoResponse {
    "Hello, world!"
}
