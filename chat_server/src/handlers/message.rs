use axum::{
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use tokio::fs;
use tracing::{info, warn};

use crate::{AppError, AppState, ChatFile, CreateMessage, ErrorOutPut, ListMessages};
use chat_core::{Chat, User};

/// List all messages in the chat.
#[utoipa::path(
    get,
    path = "/api/chats/:id/messages",
    responses(
        (status = 200, description = "List all messages", body = Vec<Chat>),
    ),
    security(("token" = []))
)]
pub(crate) async fn list_message_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Query(input): Query<ListMessages>, // Query is used to extract query parameters put in the URL
) -> Result<impl IntoResponse, AppError> {
    let messages = state.list_messages(input, id).await?;
    Ok(Json(messages))
}

/// Create a new message in the chat.
#[utoipa::path(
    post,
    path = "/api/chats/:id/messages",
    responses(
        (status = 201, description = "Message created", body = Chat),
    ),
    security(("token" = []))
)]
pub(crate) async fn send_message_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<CreateMessage>,
) -> Result<impl IntoResponse, AppError> {
    let msg = state.create_message(input, id, user.id as u64).await?;
    Ok((StatusCode::CREATED, Json(msg)))
}
/// file_handler is used to serve files from the server.
#[utoipa::path(
    get,
    path = "/api/ws/:ws_id/files/*path",
    responses(
        (status = 200, description = "File found", body = Vec<u8>),
        (status = 404, description = "File not found", body = ErrorOutPut),
    ),
    security(("token" = []))
)]
pub(crate) async fn file_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path((ws_id, path)): Path<(u64, String)>,
) -> Result<impl IntoResponse, AppError> {
    if user.ws_id != ws_id as i64 {
        return Err(AppError::NotFound(
            "File doesn't exist or you don't have permission".to_string(),
        ));
    }

    let base_dir = state.config.server.base_dir.join(ws_id.to_string());
    let path = base_dir.join(path);
    if !path.exists() {
        return Err(AppError::NotFound("File doesn't exist".to_string()));
    }

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    // TODO: streaming
    let body = fs::read(path).await?;
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", mime.to_string().parse().unwrap());

    Ok((headers, body))
}

/// upload_handler is used to upload files to the server.
#[utoipa::path(
    post,
    path = "/api/ws/:ws_id/files",
    responses(
        (status = 200, description = "File uploaded", body = Vec<String>),
    ),
    security(("token" = []))
)]
pub(crate) async fn upload_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id as u64;
    let base_dir = &state.config.server.base_dir;
    let mut files = vec![];
    while let Some(field) = multipart.next_field().await.unwrap() {
        let filename = field.file_name().map(|name| name.to_string());
        let (Some(filename), Ok(data)) = (filename, field.bytes().await) else {
            warn!("Failed to read multipart filed",);
            continue;
        };

        let file = ChatFile::new(ws_id, &filename, &data);
        let path = file.path(base_dir);
        if path.exists() {
            info!("File {} already exists: {:?}", filename, path);
        } else {
            fs::create_dir_all(path.parent().expect("file path parent should exists")).await?;
            fs::write(&path, data).await?;
        }

        files.push(file.url());
    }

    Ok(Json(files))
}
