use std::sync::Arc;

use axum::extract::FromRef;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::app::triage::workflow::Triage;

#[derive(Clone)]
pub struct AppState {
    pub triage: Arc<Triage>,
    pub webhook_secret: Arc<str>,
    pub tasks: TaskTracker,
    pub shutdown: CancellationToken,
}

impl FromRef<AppState> for Arc<str> {
    fn from_ref(state: &AppState) -> Self { state.webhook_secret.clone() }
}
