use crate::config::ConfigError;
use crate::domains::github::GitHubError;
use crate::domains::llm::LlmError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("configuration loading failed")]
    Config(#[from] ConfigError),

    #[error("github connector initialization failed")]
    GitHub(#[from] GitHubError),

    #[error("llm connector initialization failed")]
    Llm(#[from] LlmError),

    #[error("server io failure")]
    Io,
}
