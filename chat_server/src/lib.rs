mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;

use anyhow::Context;
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use handlers::*;
use sqlx::PgPool;
use std::{fmt, ops::Deref, sync::Arc};
use tokio::fs;
use utils::{DecodingKey, EncodingKey};

pub use config::AppConfig;
pub use error::{AppError, ErrorOutPut};
pub use middlewares::{set_layers, verify_token};
pub use models::*;

#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) ek: EncodingKey,
    pub(crate) dk: DecodingKey,
    pub(crate) pool: PgPool,
}

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;

    let api = Router::new()
        .route("/users", get(list_chat_users_handler))
        .route("/chats", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chats/:id",
            get(get_chat_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chats/:id/messages", get(list_message_handler))
        .route("/upload", post(upload_handler))
        .route("/files/:ws_id/*path", get(file_handler))
        .layer(from_fn_with_state(state.clone(), verify_token))
        // routes don't need token
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);

    Ok(set_layers(app))
}

// state.config => state.inner.config
impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        fs::create_dir_all(&config.server.base_dir)
            .await
            .context("create base_dir failed")?;
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .context("connect to database failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                ek,
                dk,
                pool,
            }),
        })
    }
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod test_util {
    use std::path::Path;

    use super::*;
    use sqlx::{Executor, PgPool};
    use sqlx_db_tester::TestPg;

    impl AppState {
        pub async fn new_for_test(config: AppConfig) -> Result<(TestPg, Self), AppError> {
            let ek = EncodingKey::load(&config.auth.sk)?;
            let dk = DecodingKey::load(&config.auth.pk)?;
            let pos = config.server.db_url.rfind('/').expect("invalid db_url");
            let server_url = &config.server.db_url[..pos];
            let (tdb, pool) = get_test_pool(Some(server_url)).await;
            let state = Self {
                inner: Arc::new(AppStateInner {
                    config,
                    ek,
                    dk,
                    pool,
                }),
            };
            Ok((tdb, state))
        }
    }

    pub async fn get_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
        let url = match url {
            Some(url) => url.to_string(),
            None => "postgres://postgres:password@localhost:5432".to_string(),
        };
        let tdb = TestPg::new(url, Path::new("../migrations"));
        let pool = tdb.get_pool().await;

        // run prepareed sql to insert data  include_str! macro to read file
        let sql = include_str!("../fixtures/test.sql").split(";");
        // run sql and use sqlx transaction
        let mut ts = pool.begin().await.expect("begin transaction failed");
        for q in sql {
            if q.trim().is_empty() {
                continue;
            }
            ts.execute(q).await.expect("execute sql failed");
        }
        ts.commit().await.expect("commit transaction failed");

        (tdb, pool)
    }
}
