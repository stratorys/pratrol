use crate::ports::github::GitHubError;

#[derive(Debug, thiserror::Error)]
pub enum TriageError {
    #[error(transparent)]
    GitHub(#[from] GitHubError),

    #[error("pull request has no commits")]
    NoCommits,
}
