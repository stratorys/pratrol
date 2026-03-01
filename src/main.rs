mod api;
mod config;
mod connectors;
mod domains;
mod ports;

use std::sync::Arc;

use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::connectors::github::GitHubConnector;
use crate::connectors::mistral::MistralConnector;
use crate::domains::triage::service::TriageService;

#[derive(Clone)]
pub struct AppState {
    pub triage_service: Arc<TriageService<GitHubConnector, MistralConnector>>,
    pub webhook_secret: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(error) => {
            error!(message = "Failed to load configuration.", %error);
            std::process::exit(1);
        }
    };

    let github = match GitHubConnector::new(config.github_app_id, &config.github_private_key) {
        Ok(connector) => Arc::new(connector),
        Err(error) => {
            error!(message = "Failed to initialize GitHub connector.", %error);
            std::process::exit(1);
        }
    };

    let mistral = match MistralConnector::new(config.mistral_api_key) {
        Ok(connector) => Arc::new(connector),
        Err(error) => {
            error!(message = "Failed to initialize Mistral connector.", %error);
            std::process::exit(1);
        }
    };

    let triage_service = Arc::new(TriageService::new(github, mistral));

    let state = AppState {
        triage_service,
        webhook_secret: config.github_webhook_secret,
    };

    let app = api::router().with_state(state);

    let listener = match tokio::net::TcpListener::bind(config.listen_addr).await {
        Ok(listener) => listener,
        Err(error) => {
            error!(message = "Failed to bind listener.", %error, addr = %config.listen_addr);
            std::process::exit(1);
        }
    };

    info!(message = "Server started.", addr = %config.listen_addr);

    if let Err(error) = axum::serve(listener, app).await {
        error!(message = "Server error.", %error);
        std::process::exit(1);
    }
}
