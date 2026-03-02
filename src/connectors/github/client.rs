use async_trait::async_trait;
use chrono::Utc;
use octocrab::models::pulls::ReviewAction;

use super::InstalledClient;
use crate::ports::github::{
    CommitInfo,
    GitHubClient,
    GitHubError,
    UserInfo,
};

const DIFF_MAX_CHARS: usize = 30_000;
const COMMIT_MESSAGE_MAX_CHARS: usize = 500;

#[async_trait]
impl GitHubClient for InstalledClient {
    async fn fetch_user(
        &self,
        login: &str,
    ) -> Result<UserInfo, GitHubError> {
        let profile = self.octocrab.users(login).profile().await?;

        let age_days = (Utc::now() - profile.created_at).num_days().max(0) as u32;

        let public_repos = u32::try_from(profile.public_repos).unwrap_or(u32::MAX);
        let followers = u32::try_from(profile.followers).unwrap_or(u32::MAX);

        Ok(UserInfo {
            account_age_days: age_days,
            public_repos,
            followers,
        })
    }

    async fn fetch_events_count(
        &self,
        login: &str,
    ) -> Result<u32, GitHubError> {
        let route = format!("/users/{login}/events/public?per_page=100");
        let events: Vec<serde_json::Value> = self.octocrab.get(route, None::<&()>).await?;
        Ok(events.len() as u32)
    }

    async fn fetch_orgs_count(
        &self,
        login: &str,
    ) -> Result<u32, GitHubError> {
        let route = format!("/users/{login}/orgs?per_page=100");
        let orgs: Vec<serde_json::Value> = self.octocrab.get(route, None::<&()>).await?;
        Ok(orgs.len() as u32)
    }

    async fn fetch_merged_prs(
        &self,
        login: &str,
        owner: &str,
        repo: &str,
    ) -> Result<u32, GitHubError> {
        let query = format!("is:pr is:merged author:{login} repo:{owner}/{repo}");
        self.search_issues_count(&query).await
    }

    async fn fetch_merged_prs_global(
        &self,
        login: &str,
    ) -> Result<u32, GitHubError> {
        let query = format!("is:pr is:merged author:{login}");
        self.search_issues_count(&query).await
    }

    async fn fetch_diff(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<String, GitHubError> {
        let mut diff = self.octocrab.pulls(owner, repo).get_diff(pr_number).await?;

        if diff.len() > DIFF_MAX_CHARS {
            diff.truncate(DIFF_MAX_CHARS);
        }

        Ok(diff)
    }

    async fn fetch_commits(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<Vec<CommitInfo>, GitHubError> {
        let page = self
            .octocrab
            .pulls(owner, repo)
            .pr_commits(pr_number)
            .per_page(100)
            .send()
            .await?;

        let commits: Vec<CommitInfo> = page
            .items
            .into_iter()
            .map(|repo_commit| {
                let mut message = repo_commit.commit.message;
                if message.len() > COMMIT_MESSAGE_MAX_CHARS {
                    message.truncate(COMMIT_MESSAGE_MAX_CHARS);
                }
                CommitInfo {
                    sha: repo_commit.sha,
                    message,
                }
            })
            .collect();

        Ok(commits)
    }

    #[allow(
        deprecated,
        reason = "octocrab has no non-deprecated path to create_review"
    )]
    async fn post_review(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        commit_sha: &str,
        body: &str,
    ) -> Result<(), GitHubError> {
        self.octocrab
            .pulls(owner, repo)
            .pull_number(pr_number)
            .reviews()
            .create_review(commit_sha, body, ReviewAction::Comment, vec![])
            .await?;

        Ok(())
    }

    async fn add_labels(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        labels: Vec<String>,
    ) -> Result<(), GitHubError> {
        self.octocrab
            .issues(owner, repo)
            .add_labels(pr_number, &labels)
            .await?;

        Ok(())
    }

    async fn ensure_label(
        &self,
        owner: &str,
        repo: &str,
        name: String,
        color: String,
        description: String,
    ) -> Result<(), GitHubError> {
        let result = self.octocrab.issues(owner, repo).get_label(&name).await;

        match result {
            Ok(_) => Ok(()),
            Err(octocrab::Error::GitHub {
                ref source, ..
            }) if source.status_code == http::StatusCode::NOT_FOUND => {
                self.octocrab
                    .issues(owner, repo)
                    .create_label(&name, &color, &description)
                    .await?;
                Ok(())
            }
            Err(error) => Err(GitHubError::Api(error)),
        }
    }
}

impl InstalledClient {
    async fn search_issues_count(
        &self,
        query: &str,
    ) -> Result<u32, GitHubError> {
        let page = self
            .octocrab
            .search()
            .issues_and_pull_requests(query)
            .per_page(1)
            .send()
            .await?;

        let count = page.total_count.unwrap_or(0);
        let count_u32 = u32::try_from(count).unwrap_or(u32::MAX);

        Ok(count_u32)
    }
}
