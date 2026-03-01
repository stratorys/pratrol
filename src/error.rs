use crate::config::ConfigError;
use crate::ports::github::GitHubError;
use crate::ports::mistral::MistralError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    GitHub(#[from] GitHubError),

    #[error(transparent)]
    Mistral(#[from] MistralError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
