use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{AppError, AppState, CreateUser, ErrorOutPut, SigninUser};

#[derive(Debug, Serialize, ToSchema, Deserialize)]
pub struct AuthOutPut {
    pub token: String,
}

/// Create a new user in the chat system with email, password workspace and full name.
///
/// - If the email already exists, it will return 409.
/// - Otherwise, it will return 201 with a token.
/// - If the workspace doesn't exist, it will create one.
#[utoipa::path(
    post,
    path = "/api/signup",
    responses(
        (status = 200, description = "User created", body = AuthOutPut),
    )
)]
pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.create_user(&input).await?;
    let token = state.ek.sign(user)?;
    let body = Json(AuthOutPut { token });
    Ok((StatusCode::CREATED, body))
}

/// signin a user with email and password.
///
/// - If users exist and the password is correct, it will return 200 with a token.
/// - If the email doesn't exist or the password is incorrect, it will return 403.
#[utoipa::path(
    post,
    path = "/api/signin",
    responses(
        (status = 200, description = "User signed in", body = AuthOutPut),
        (status = 403, description = "Invalid email or password", body = ErrorOutPut),
    )
)]
pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.verify_user(&input).await?;
    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            Ok((StatusCode::OK, Json(AuthOutPut { token })).into_response())
        }
        None => {
            let body = Json(ErrorOutPut::new("Invalid email or password"));
            Ok((StatusCode::FORBIDDEN, body).into_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn signup_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("none", "tian yu", "yut@acme.org", "123456");
        let ret = signup_handler(State(state), Json(input))
            .await?
            .into_response();
        assert_eq!(ret.status(), StatusCode::CREATED);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: AuthOutPut = serde_json::from_slice(&body)?;
        assert_ne!(ret.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn signup_duplicate_user_should_409() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("acme", "Yu Tian", "yu@acme.org", "123456");
        let ret = signup_handler(State(state), Json(input))
            .await
            .into_response();
        assert_eq!(ret.status(), StatusCode::CONFLICT);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: ErrorOutPut = serde_json::from_slice(&body)?;
        assert_eq!(ret.error, "email already exists: yu@acme.org");
        Ok(())
    }

    #[tokio::test]
    async fn signin_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let email = "yu@acme.org";
        let password = "123456";

        let input = SigninUser::new(email, password);
        let ret = signin_handler(State(state), Json(input))
            .await?
            .into_response();
        assert_eq!(ret.status(), StatusCode::OK);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: AuthOutPut = serde_json::from_slice(&body)?;
        assert_ne!(ret.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn signin_non_exist_user_should_403() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let email = "yu1@acme.org";
        let password = "123456";
        let input = SigninUser::new(email, password);
        let ret = signin_handler(State(state), Json(input))
            .await
            .into_response();
        assert_eq!(ret.status(), StatusCode::FORBIDDEN);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: ErrorOutPut = serde_json::from_slice(&body)?;
        assert_eq!(ret.error, "Invalid email or password");
        Ok(())
    }
}
