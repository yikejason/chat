mod config;
mod error;
mod notify;
mod sse;

use std::{ops::Deref, sync::Arc};

use anyhow::Result;
use axum::{
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chat_core::{
    middlewares::{verify_token, TokenVerify},
    DecodingKey, User,
};
use dashmap::DashMap;
use sse::*;
use tokio::sync::broadcast;

pub use config::AppConfig;
pub use error::AppError;
pub use notify::AppEvent;

pub type UserMap = Arc<DashMap<u64, broadcast::Sender<Arc<AppEvent>>>>; // Arc is used to share the DashMap between multiple threads

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub config: AppConfig,
    pub users: UserMap,
    dk: DecodingKey,
}

const INDEX_HTML: &str = include_str!("../index.html");

pub async fn get_router(config: AppConfig) -> anyhow::Result<Router> {
    let state = AppState::new(config);
    notify::setup_pg_listener(state.clone()).await?;
    let app = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/", get(index_handler))
        .with_state(state.clone());
    Ok(app)
}

pub(crate) async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TokenVerify for AppState {
    type Error = AppError;

    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let dk = DecodingKey::load(&config.auth.pk).expect("load pk failed");
        let users = Arc::new(DashMap::new());
        Self(Arc::new(AppStateInner { config, users, dk }))
    }
}
