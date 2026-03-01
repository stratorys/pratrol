pub mod error;
pub mod webhook;

use axum::Router;
use axum::routing::post;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/webhook/github", post(webhook::handler::handle_webhook))
}
