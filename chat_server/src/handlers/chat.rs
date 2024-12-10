use axum::{response::IntoResponse, Extension};
use tracing::info;

use crate::User;

pub(crate) async fn list_chat_handler(Extension(user): Extension<User>) -> impl IntoResponse {
    info!("{:?}", user);
    "list_chat"
}

pub(crate) async fn create_chat_handler() -> impl IntoResponse {
    "create_chat"
}

pub(crate) async fn update_chat_handler() -> impl IntoResponse {
    "update_chat"
}

pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    "delete_chat"
}
