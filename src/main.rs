mod agent;
mod api;
mod app;
mod config;
mod connectors;
mod domains;
mod error;
mod sanitize;

use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use rustls::crypto::CryptoProvider;
use rustls::crypto::ring::default_provider;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{
    error,
    info,
    warn,
};
use tracing_subscriber::EnvFilter;

use crate::agent::harness::Harness;
use crate::api::state::AppState;
use crate::app::triage::workflow::Triage;
use crate::config::Config;
#[cfg(feature = "gemma")]
use crate::connectors::gemma::connector::GemmaConnector;
use crate::connectors::github::connector::GitHubConnector;
#[cfg(feature = "mistral")]
use crate::connectors::mistral::connector::MistralConnector;
use crate::domains::llm::traits::Llm;
use crate::error::AppError;

#[cfg(all(feature = "mistral", feature = "gemma"))]
compile_error!("features `mistral` and `gemma` are mutually exclusive");
#[cfg(not(any(feature = "mistral", feature = "gemma")))]
compile_error!("enable exactly one LLM engine: `mistral` or `gemma`");

const SHUTDOWN_GRACE: Duration = Duration::from_secs(25);

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

    #[cfg(feature = "mistral")]
    let llm: Arc<dyn Llm> = Arc::new(MistralConnector::new(&config)?);
    #[cfg(feature = "gemma")]
    let llm: Arc<dyn Llm> = Arc::new(GemmaConnector::new(&config)?);

    let triage = Arc::new(Triage::new(github, Harness::new(llm)));

    let tasks = TaskTracker::new();
    let shutdown_token = CancellationToken::new();

    let state = AppState {
        triage,
        webhook_secret: config.github_webhook_secret.into(),
        tasks: tasks.clone(),
        shutdown: shutdown_token.clone(),
    };

    let app = api::router::router().with_state(state);

    let listener = tokio::net::TcpListener::bind(config.listen_addr)
        .await
        .map_err(|error| {
            error!(message = "Failed to bind listener.", %error, addr = %config.listen_addr);
            AppError::Io
        })?;

    info!(message = "Server started.", addr = %config.listen_addr);

    let shutdown_signal = async {
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
        .with_graceful_shutdown(shutdown_signal)
        .await
        .map_err(|error| {
            error!(message = "Server failed while serving requests.", %error);
            AppError::Io
        })?;

    tasks.close();
    info!(message = "HTTP drained, waiting for in-flight triage tasks.");
    if tokio::time::timeout(SHUTDOWN_GRACE, tasks.wait())
        .await
        .is_err()
    {
        warn!(message = "Grace period exceeded, cancelling in-flight triage tasks.");
        shutdown_token.cancel();
        tasks.wait().await;
    }

    info!(message = "Server stopped.");

    Ok(())
}
