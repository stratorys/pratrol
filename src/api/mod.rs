pub mod error;
pub mod health;
pub mod state;
pub mod webhook;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{
    get,
    post,
};

use crate::api::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::handle_health))
        .route("/webhook/github", post(webhook::handler::handle_webhook))
        .layer(DefaultBodyLimit::max(256 * 1024))
}
