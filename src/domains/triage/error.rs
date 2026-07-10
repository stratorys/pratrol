use crate::domains::github::error::GitHubError;

#[derive(Debug, thiserror::Error)]
pub enum TriageError {
    #[error("github operation failed")]
    GitHub(#[from] GitHubError),

    #[error("pull request has no commits")]
    NoCommits,
}
