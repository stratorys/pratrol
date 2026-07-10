#[derive(Debug, thiserror::Error)]
pub enum GitHubError {
    #[error("github api request failed")]
    Api,

    #[error("github app credential setup failed")]
    Jwt,
}
