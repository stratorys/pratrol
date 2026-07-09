use std::sync::Arc;

use crate::app::triage::service::TriageService;

#[derive(Clone)]
pub struct AppState {
    pub triage_service: Arc<TriageService>,
    pub webhook_secret: Arc<str>,
}
