mod api;
mod config;
mod connectors;
mod domains;
mod error;
mod sanitize;

use std::process::ExitCode;
use std::sync::Arc;

use rustls::crypto::CryptoProvider;
use rustls::crypto::ring::default_provider;
use tracing::{
    error,
    info,
};
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::connectors::github::GitHubConnector;
use crate::connectors::mistral::MistralConnector;
use crate::domains::triage::service::TriageService;
use crate::error::AppError;

#[derive(Clone)]
pub struct AppState {
    pub triage_service: Arc<TriageService<GitHubConnector, MistralConnector>>,
    pub webhook_secret: Arc<str>,
}

#[tokio::main]
async fn main() -> ExitCode {
    CryptoProvider::install_default(default_provider())
        .map_err(|_| "Failed to install default CryptoProvider.")
        .ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            error!(message = "Fatal error.", %error);
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), AppError> {
    let config = Config::from_env()?;

    let github =
        Arc::new(GitHubConnector::new(config.github_app_id, &config.github_private_key).await?);

    let mistral = Arc::new(MistralConnector::new(config.clone())?);

    let triage_service = Arc::new(TriageService::new(github, mistral));

    let state = AppState {
        triage_service,
        webhook_secret: config.github_webhook_secret.into(),
    };

    let app = api::router().with_state(state);

    let listener = tokio::net::TcpListener::bind(config.listen_addr).await?;

    info!(message = "Server started.", addr = %config.listen_addr);

    let shutdown = async {
        let ctrl_c = tokio::signal::ctrl_c();
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler.");
        tokio::select! {
            _ = ctrl_c => {}
            _ = sigterm.recv() => {}
        }
        info!(message = "Shutdown signal received, draining connections.");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;

    info!(message = "Server stopped.");

    Ok(())
}
