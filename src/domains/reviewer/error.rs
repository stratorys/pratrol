use crate::ports::github::GitHubError;

#[derive(Debug, thiserror::Error)]
pub enum ReviewerError {
    #[error(transparent)]
    GitHub(#[from] GitHubError),
}
