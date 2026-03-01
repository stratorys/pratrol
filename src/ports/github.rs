use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum GitHubError {
    #[error(transparent)]
    Api(#[from] octocrab::Error),

    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    TimeParse(#[from] chrono::ParseError),

    #[error("unexpected response status")]
    UnexpectedStatus,
}

pub struct UserInfo {
    pub account_age_days: u32,
    pub public_repos: u32,
    pub followers: u32,
}

#[async_trait]
pub trait GitHubApp: Send + Sync {
    type Client: GitHubClient;

    async fn installation_client(
        &self,
        installation_id: u64,
    ) -> Result<Self::Client, GitHubError>;
}

#[async_trait]
pub trait GitHubClient: Send + Sync {
    async fn fetch_user(
        &self,
        login: &str,
    ) -> Result<UserInfo, GitHubError>;

    async fn fetch_events_count(
        &self,
        login: &str,
    ) -> Result<u32, GitHubError>;

    async fn fetch_orgs_count(
        &self,
        login: &str,
    ) -> Result<u32, GitHubError>;

    async fn fetch_merged_prs(
        &self,
        login: &str,
        owner: &str,
        repo: &str,
    ) -> Result<u32, GitHubError>;

    async fn fetch_merged_prs_global(
        &self,
        login: &str,
    ) -> Result<u32, GitHubError>;

    async fn fetch_diff(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<String, GitHubError>;

    async fn fetch_commits(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<Vec<String>, GitHubError>;

    async fn post_review(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        body: &str,
    ) -> Result<(), GitHubError>;

    async fn add_labels(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        labels: Vec<String>,
    ) -> Result<(), GitHubError>;
}
