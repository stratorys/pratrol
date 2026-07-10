use async_trait::async_trait;
use chrono::Utc;
use octocrab::Page;
use octocrab::models::pulls::ReviewAction;
use serde::Deserialize;
use tracing::error;

use super::connector::InstalledClient;
use crate::domains::github::entity::{
    CommitInfo,
    RejectedPrInfo,
    RejectedPrSearchResult,
    UserInfo,
};
use crate::domains::github::error::GitHubError;
use crate::domains::github::traits::GitHubClient;

const DIFF_MAX_CHARS: usize = 30_000;
const COMMIT_MESSAGE_MAX_CHARS: usize = 500;

const MAX_REVIEW_PAGES: u32 = 3;

#[derive(Deserialize)]
struct PublicEvent {
    _id: String,
}

#[derive(Deserialize)]
struct OrgItem {
    _login: String,
}

#[async_trait]
impl GitHubClient for InstalledClient {
    async fn fetch_user(
        &self,
        login: &str,
    ) -> Result<UserInfo, GitHubError> {
        let profile = self
            .octocrab
            .users(login)
            .profile()
            .await
            .map_err(|error| {
                error!(message = "Failed to fetch user profile.", %error, login);
                GitHubError::Api
            })?;

        let age_days = u32::try_from((Utc::now() - profile.created_at).num_days().max(0))
            .map_err(|_| GitHubError::InvalidResponse)?;

        let public_repos =
            u32::try_from(profile.public_repos).map_err(|_| GitHubError::InvalidResponse)?;
        let followers =
            u32::try_from(profile.followers).map_err(|_| GitHubError::InvalidResponse)?;

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
        let page = self.get_user_public_events(login).await.map_err(|error| {
            error!(message = "Failed to fetch user public events.", %error, login);
            GitHubError::Api
        })?;
        let items_len =
            u32::try_from(page.items.len()).map_err(|_| GitHubError::InvalidResponse)?;
        let total = match page.number_of_pages() {
            Some(n) if n > 1 => (n - 1) * 100 + items_len,
            _ => items_len,
        };
        Ok(total)
    }

    async fn fetch_orgs_count(
        &self,
        login: &str,
    ) -> Result<u32, GitHubError> {
        let orgs = self.get_user_orgs(login).await.map_err(|error| {
            error!(message = "Failed to fetch user organizations.", %error, login);
            GitHubError::Api
        })?;
        u32::try_from(orgs.len()).map_err(|_| GitHubError::InvalidResponse)
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
        let diff = self
            .octocrab
            .pulls(owner, repo)
            .get_diff(pr_number)
            .await
            .map_err(|error| {
                error!(
                    message = "Failed to fetch pull request diff.",
                    %error,
                    owner,
                    repo,
                    pr_number,
                );
                GitHubError::Api
            })?;

        Ok(truncate_chars(&diff, DIFF_MAX_CHARS))
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
            .await
            .map_err(|error| {
                error!(
                    message = "Failed to fetch pull request commits.",
                    %error,
                    owner,
                    repo,
                    pr_number,
                );
                GitHubError::Api
            })?;

        let commits: Vec<CommitInfo> = page
            .items
            .into_iter()
            .map(|repo_commit| CommitInfo {
                sha: repo_commit.sha,
                message: truncate_chars(&repo_commit.commit.message, COMMIT_MESSAGE_MAX_CHARS),
            })
            .collect();

        Ok(commits)
    }

    async fn has_pratrol_review(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<bool, GitHubError> {
        for page_number in 1..=MAX_REVIEW_PAGES {
            let page = self
                .octocrab
                .pulls(owner, repo)
                .list_reviews(pr_number)
                .per_page(100)
                .page(page_number)
                .send()
                .await
                .map_err(|error| {
                    error!(
                        message = "Failed to list pull request reviews.",
                        %error,
                        owner,
                        repo,
                        pr_number,
                        page_number,
                    );
                    GitHubError::Api
                })?;
            if page
                .items
                .iter()
                .any(|r| r.user.as_ref().is_some_and(|u| u.login == self.bot_login))
            {
                return Ok(true);
            }
            if page.items.len() < 100 {
                return Ok(false);
            }
        }
        Ok(false)
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
            .await
            .map_err(|error| {
                error!(
                    message = "Failed to post review.",
                    %error,
                    owner,
                    repo,
                    pr_number,
                );
                GitHubError::Api
            })?;

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
            .await
            .map_err(|error| {
                error!(
                    message = "Failed to add labels.",
                    %error,
                    owner,
                    repo,
                    pr_number,
                );
                GitHubError::Api
            })?;

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
                create_label(self, owner, repo, &name, &color, &description).await
            }
            Err(error) => {
                error!(
                    message = "Failed to fetch label.",
                    %error,
                    owner,
                    repo,
                    label = %name,
                );
                Err(GitHubError::Api)
            }
        }
    }

    async fn search_rejected_prs_by_author(
        &self,
        login: &str,
        owner: &str,
        repo: &str,
    ) -> Result<RejectedPrSearchResult, GitHubError> {
        let query = format!("is:pr is:closed is:unmerged author:{login} repo:{owner}/{repo}");
        self.search_rejected_prs(&query).await
    }

    async fn search_rejected_prs_by_title(
        &self,
        keywords: &str,
        owner: &str,
        repo: &str,
    ) -> Result<RejectedPrSearchResult, GitHubError> {
        let query = format!("is:pr is:closed is:unmerged repo:{owner}/{repo} \"{keywords}\"");
        self.search_rejected_prs(&query).await
    }

    async fn search_rejected_prs_by_author_global(
        &self,
        login: &str,
    ) -> Result<u32, GitHubError> {
        let query = format!("is:pr is:closed is:unmerged author:{login}");
        self.search_issues_count(&query).await
    }
}

impl InstalledClient {
    // octocrab 0.49 does not expose GET /users/{login}/events/public
    async fn get_user_public_events(
        &self,
        login: &str,
    ) -> Result<Page<PublicEvent>, octocrab::Error> {
        self.octocrab
            .get(
                format!("/users/{login}/events/public"),
                Some(&[("per_page", "100")]),
            )
            .await
    }

    // octocrab 0.49 does not expose GET /users/{login}/orgs
    async fn get_user_orgs(
        &self,
        login: &str,
    ) -> Result<Vec<OrgItem>, octocrab::Error> {
        self.octocrab
            .get(format!("/users/{login}/orgs"), Some(&[("per_page", "100")]))
            .await
    }

    async fn search_rejected_prs(
        &self,
        query: &str,
    ) -> Result<RejectedPrSearchResult, GitHubError> {
        const MAX_RESULTS: usize = 5;

        let page_size = u8::try_from(MAX_RESULTS).map_err(|_| GitHubError::InvalidResponse)?;
        let page = self
            .octocrab
            .search()
            .issues_and_pull_requests(query)
            .per_page(page_size)
            .send()
            .await
            .map_err(|error| {
                error!(message = "Failed to search rejected pull requests.", %error, query);
                GitHubError::Api
            })?;

        let total_count = page
            .total_count
            .ok_or(GitHubError::InvalidResponse)
            .and_then(|count| u32::try_from(count).map_err(|_| GitHubError::InvalidResponse))?;

        let items: Vec<RejectedPrInfo> = page
            .items
            .into_iter()
            .take(MAX_RESULTS)
            .map(|issue| RejectedPrInfo {
                number: issue.number,
                title: issue.title,
                html_url: issue.html_url.to_string(),
            })
            .collect();

        Ok(RejectedPrSearchResult {
            total_count,
            items,
        })
    }

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
            .await
            .map_err(|error| {
                error!(message = "Failed to search issues count.", %error, query);
                GitHubError::Api
            })?;

        let count = page.total_count.ok_or(GitHubError::InvalidResponse)?;
        u32::try_from(count).map_err(|_| GitHubError::InvalidResponse)
    }
}

async fn create_label(
    client: &InstalledClient,
    owner: &str,
    repo: &str,
    name: &str,
    color: &str,
    description: &str,
) -> Result<(), GitHubError> {
    let result = client
        .octocrab
        .issues(owner, repo)
        .create_label(name, color, description)
        .await;

    match result {
        Ok(_) => Ok(()),
        Err(octocrab::Error::GitHub {
            ref source, ..
        }) if source.status_code == http::StatusCode::UNPROCESSABLE_ENTITY => Ok(()),
        Err(error) => {
            error!(
                message = "Failed to create label.",
                %error,
                owner,
                repo,
                label = %name,
            );
            Err(GitHubError::Api)
        }
    }
}

fn truncate_chars(
    input: &str,
    max_chars: usize,
) -> String {
    input.chars().take(max_chars).collect()
}
