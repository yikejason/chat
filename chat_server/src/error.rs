use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Failed to connect to the database")]
    SqlxError(#[from] sqlx::Error),

    #[error("Failed to hash password")]
    Argon2Error(#[from] argon2::password_hash::Error),
}
