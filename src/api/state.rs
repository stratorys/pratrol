use std::sync::Arc;

use crate::app::triage::workflow::Triage;

#[derive(Clone)]
pub struct AppState {
    pub triage: Arc<Triage>,
    pub webhook_secret: Arc<str>,
}
