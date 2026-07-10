use std::sync::Arc;

use async_trait::async_trait;

pub use super::entity::{
    CommitInfo,
    RejectedPrSearchResult,
    UserInfo,
};
pub use super::error::GitHubError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait GitHubApp: Send + Sync {
    async fn installation_client(
        &self,
        installation_id: u64,
    ) -> Result<Arc<dyn GitHubClient>, GitHubError>;
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

    async fn has_pratrol_review(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<bool, GitHubError>;

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

    async fn search_rejected_prs_by_author(
        &self,
        login: &str,
        owner: &str,
        repo: &str,
    ) -> Result<RejectedPrSearchResult, GitHubError>;

    async fn search_rejected_prs_by_title(
        &self,
        keywords: &str,
        owner: &str,
        repo: &str,
    ) -> Result<RejectedPrSearchResult, GitHubError>;

    async fn search_rejected_prs_by_author_global(
        &self,
        login: &str,
    ) -> Result<u32, GitHubError>;
}

#[async_trait]
pub trait PullRequestReader: Send + Sync {
    async fn has_pratrol_review(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<bool, GitHubError>;

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
}

#[async_trait]
pub trait ContributorReader: Send + Sync {
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
}

#[async_trait]
pub trait HistoryReader: Send + Sync {
    async fn search_rejected_prs_by_author(
        &self,
        login: &str,
        owner: &str,
        repo: &str,
    ) -> Result<RejectedPrSearchResult, GitHubError>;

    async fn search_rejected_prs_by_title(
        &self,
        keywords: &str,
        owner: &str,
        repo: &str,
    ) -> Result<RejectedPrSearchResult, GitHubError>;

    async fn search_rejected_prs_by_author_global(
        &self,
        login: &str,
    ) -> Result<u32, GitHubError>;
}

#[async_trait]
pub trait ReviewPublisher: Send + Sync {
    async fn post_review(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        commit_sha: &str,
        body: &str,
    ) -> Result<(), GitHubError>;
}

#[async_trait]
pub trait LabelManager: Send + Sync {
    async fn ensure_label(
        &self,
        owner: &str,
        repo: &str,
        name: String,
        color: String,
        description: String,
    ) -> Result<(), GitHubError>;

    async fn add_labels(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        labels: Vec<String>,
    ) -> Result<(), GitHubError>;
}

#[async_trait]
impl<T: GitHubClient + ?Sized> PullRequestReader for T {
    async fn has_pratrol_review(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<bool, GitHubError> {
        GitHubClient::has_pratrol_review(self, owner, repo, pr_number).await
    }

    async fn fetch_diff(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<String, GitHubError> {
        GitHubClient::fetch_diff(self, owner, repo, pr_number).await
    }

    async fn fetch_commits(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<Vec<CommitInfo>, GitHubError> {
        GitHubClient::fetch_commits(self, owner, repo, pr_number).await
    }
}

#[async_trait]
impl<T: GitHubClient + ?Sized> ContributorReader for T {
    async fn fetch_user(
        &self,
        login: &str,
    ) -> Result<UserInfo, GitHubError> {
        GitHubClient::fetch_user(self, login).await
    }

    async fn fetch_events_count(
        &self,
        login: &str,
    ) -> Result<u32, GitHubError> {
        GitHubClient::fetch_events_count(self, login).await
    }

    async fn fetch_orgs_count(
        &self,
        login: &str,
    ) -> Result<u32, GitHubError> {
        GitHubClient::fetch_orgs_count(self, login).await
    }

    async fn fetch_merged_prs(
        &self,
        login: &str,
        owner: &str,
        repo: &str,
    ) -> Result<u32, GitHubError> {
        GitHubClient::fetch_merged_prs(self, login, owner, repo).await
    }

    async fn fetch_merged_prs_global(
        &self,
        login: &str,
    ) -> Result<u32, GitHubError> {
        GitHubClient::fetch_merged_prs_global(self, login).await
    }
}

#[async_trait]
impl<T: GitHubClient + ?Sized> HistoryReader for T {
    async fn search_rejected_prs_by_author(
        &self,
        login: &str,
        owner: &str,
        repo: &str,
    ) -> Result<RejectedPrSearchResult, GitHubError> {
        GitHubClient::search_rejected_prs_by_author(self, login, owner, repo).await
    }

    async fn search_rejected_prs_by_title(
        &self,
        keywords: &str,
        owner: &str,
        repo: &str,
    ) -> Result<RejectedPrSearchResult, GitHubError> {
        GitHubClient::search_rejected_prs_by_title(self, keywords, owner, repo).await
    }

    async fn search_rejected_prs_by_author_global(
        &self,
        login: &str,
    ) -> Result<u32, GitHubError> {
        GitHubClient::search_rejected_prs_by_author_global(self, login).await
    }
}

#[async_trait]
impl<T: GitHubClient + ?Sized> ReviewPublisher for T {
    async fn post_review(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        commit_sha: &str,
        body: &str,
    ) -> Result<(), GitHubError> {
        GitHubClient::post_review(self, owner, repo, pr_number, commit_sha, body).await
    }
}

#[async_trait]
impl<T: GitHubClient + ?Sized> LabelManager for T {
    async fn ensure_label(
        &self,
        owner: &str,
        repo: &str,
        name: String,
        color: String,
        description: String,
    ) -> Result<(), GitHubError> {
        GitHubClient::ensure_label(self, owner, repo, name, color, description).await
    }

    async fn add_labels(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        labels: Vec<String>,
    ) -> Result<(), GitHubError> {
        GitHubClient::add_labels(self, owner, repo, pr_number, labels).await
    }
}
