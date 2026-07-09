use crate::config::ConfigError;
use crate::domains::github::GitHubError;
use crate::domains::llm::LlmError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    GitHub(#[from] GitHubError),

    #[error(transparent)]
    Llm(#[from] LlmError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
