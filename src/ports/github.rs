use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum GitHubError {
    #[error(transparent)]
    Api(#[from] octocrab::Error),

    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

pub struct UserInfo {
    pub account_age_days: u32,
    pub public_repos: u32,
    pub followers: u32,
}

pub struct CommitInfo {
    pub sha: String,
    pub message: String,
}

#[cfg_attr(test, mockall::automock(type Client = MockGitHubClient;))]
#[async_trait]
pub trait GitHubApp: Send + Sync {
    type Client: GitHubClient;

    async fn installation_client(
        &self,
        installation_id: u64,
    ) -> Result<Self::Client, GitHubError>;
}

#[cfg_attr(test, mockall::automock)]
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
    ) -> Result<Vec<CommitInfo>, GitHubError>;

    async fn post_review(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        commit_sha: &str,
        body: &str,
    ) -> Result<(), GitHubError>;

    async fn add_labels(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        labels: Vec<String>,
    ) -> Result<(), GitHubError>;

    async fn ensure_label(
        &self,
        owner: &str,
        repo: &str,
        name: String,
        color: String,
        description: String,
    ) -> Result<(), GitHubError>;
}
