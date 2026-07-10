use crate::domains::github::error::GitHubError;

#[derive(Debug, thiserror::Error)]
pub enum TriageError {
    #[error("github operation failed")]
    GitHub(#[from] GitHubError),

    #[error("failed to render triage comment")]
    Render(#[from] askama::Error),
}
