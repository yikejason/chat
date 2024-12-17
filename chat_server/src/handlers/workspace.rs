use crate::{AppError, AppState};
use axum::{extract::State, response::IntoResponse, Extension, Json};
use chat_core::User;

/// List all users in the chat.
#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "List all users", body = Vec<User>),
    ),
    security(("token" = []))
)]
pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.fetch_chat_users(user.ws_id as _).await?;

    Ok(Json(users))
}
