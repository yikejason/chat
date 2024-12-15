use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutPut {
    pub error: String,
}

#[derive(Debug, Error)]
pub enum AppError {
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
            AppError::JwtError(_) => StatusCode::FORBIDDEN,
            AppError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(ErrorOutPut::new(self.to_string()))).into_response()
    }
}
