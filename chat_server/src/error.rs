use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct ErrorOutPut {
    pub error: String,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    ChatFileError(String),

    #[error("create message error: {0}")]
    CreateMessageError(String),

    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),

    #[error("create chat error: {0}")]
    CreateChatError(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("argon2 error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

impl ErrorOutPut {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        let status = match self {
            AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::JwtError(_) => StatusCode::FORBIDDEN,
            AppError::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            AppError::CreateChatError(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::CreateMessageError(_) => StatusCode::BAD_REQUEST,
            AppError::ChatFileError(_) => StatusCode::BAD_REQUEST,
        };
        (status, Json(ErrorOutPut::new(self.to_string()))).into_response()
    }
}
