#[derive(Debug, thiserror::Error)]
pub enum GitHubError {
    #[error(transparent)]
    Api(#[from] octocrab::Error),

    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
}
