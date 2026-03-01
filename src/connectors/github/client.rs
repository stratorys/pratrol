use async_trait::async_trait;
use chrono::Utc;
use http::header::{
    ACCEPT,
    HeaderMap,
    HeaderValue,
};
use http_body_util::BodyExt;
use percent_encoding::{
    AsciiSet,
    CONTROLS,
    utf8_percent_encode,
};
use serde::{
    Deserialize,
    Serialize,
};
use tracing::warn;

use super::InstalledClient;
use crate::ports::github::{
    GitHubClient,
    GitHubError,
    UserInfo,
};

const QUERY_ENCODE_SET: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'#').add(b'<').add(b'>');

const DIFF_MAX_CHARS: usize = 30_000;
const COMMIT_MESSAGE_MAX_CHARS: usize = 500;
const CONTRIBUTING_PATHS: &[&str] = &[
    "CONTRIBUTING.md",
    "CONTRIBUTING",
    "CONTRIBUTING.rst",
    "CONTRIBUTING.txt",
    ".github/CONTRIBUTING.md",
    "docs/CONTRIBUTING.md",
];

#[derive(Deserialize)]
struct GitHubUser {
    public_repos: u32,
    followers: u32,
    created_at: String,
}

#[derive(Deserialize)]
struct SearchResult {
    total_count: u32,
}

#[derive(Deserialize)]
struct CommitItem {
    commit: CommitDetail,
}

#[derive(Deserialize)]
struct CommitDetail {
    message: String,
}

#[derive(Serialize)]
struct CreateReviewRequest {
    body: String,
    event: String,
}

#[derive(Deserialize)]
struct CreateReviewResponse {}

#[async_trait]
impl GitHubClient for InstalledClient {
    async fn fetch_user(
        &self,
        login: &str,
    ) -> Result<UserInfo, GitHubError> {
        let route = format!("/users/{login}");
        let user: GitHubUser = self.octocrab.get(route, None::<&()>).await?;

        let created_at = chrono::DateTime::parse_from_rfc3339(&user.created_at)?;

        let age_days = (Utc::now() - created_at.with_timezone(&Utc))
            .num_days()
            .max(0) as u32;

        Ok(UserInfo {
            account_age_days: age_days,
            public_repos: user.public_repos,
            followers: user.followers,
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
        let uri = format!("/repos/{owner}/{repo}/pulls/{pr_number}");
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github.v3.diff"),
        );

        let response = self.octocrab._get_with_headers(uri, Some(headers)).await?;

        if !response.status().is_success() {
            warn!(
                message = "GitHub API returned error status for diff.",
                status = %response.status(),
            );
            return Err(GitHubError::UnexpectedStatus);
        }

        let body_bytes = response
            .into_body()
            .collect()
            .await
            .map_err(|error| {
                warn!(message = "Failed to read diff response body.", %error);
                GitHubError::UnexpectedStatus
            })?
            .to_bytes();

        let mut diff = String::from_utf8(body_bytes.to_vec()).map_err(|error| {
            warn!(
                message = "Diff response contained invalid UTF-8.",
                byte_count = error.as_bytes().len(),
            );
            GitHubError::UnexpectedStatus
        })?;

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
    ) -> Result<Vec<String>, GitHubError> {
        let route = format!("/repos/{owner}/{repo}/pulls/{pr_number}/commits?per_page=100");
        let items: Vec<CommitItem> = self.octocrab.get(route, None::<&()>).await?;
        let messages: Vec<String> = items
            .into_iter()
            .map(|item| {
                let mut message = item.commit.message;
                if message.len() > COMMIT_MESSAGE_MAX_CHARS {
                    message.truncate(COMMIT_MESSAGE_MAX_CHARS);
                }
                message
            })
            .collect();
        Ok(messages)
    }

    async fn fetch_contributing(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Option<String>, GitHubError> {
        for path in CONTRIBUTING_PATHS {
            let route = format!("/repos/{owner}/{repo}/contents/{path}");
            let mut headers = HeaderMap::new();
            headers.insert(
                ACCEPT,
                HeaderValue::from_static("application/vnd.github.raw+json"),
            );

            let response = self.octocrab._get_with_headers(route, Some(headers)).await;

            let response = match response {
                Ok(resp) => resp,
                Err(_) => continue,
            };

            if !response.status().is_success() {
                continue;
            }

            let body_bytes = response
                .into_body()
                .collect()
                .await
                .map_err(|_| GitHubError::UnexpectedStatus)?
                .to_bytes();

            let content = String::from_utf8(body_bytes.to_vec())
                .map_err(|_| GitHubError::UnexpectedStatus)?;

            return Ok(Some(content));
        }

        Ok(None)
    }

    async fn post_review(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        body: &str,
    ) -> Result<(), GitHubError> {
        let route = format!("/repos/{owner}/{repo}/pulls/{pr_number}/reviews");
        let payload = CreateReviewRequest {
            body: body.to_owned(),
            event: "COMMENT".to_owned(),
        };

        let _: CreateReviewResponse = self.octocrab.post(route, Some(&payload)).await?;

        Ok(())
    }
}

impl InstalledClient {
    async fn search_issues_count(
        &self,
        query: &str,
    ) -> Result<u32, GitHubError> {
        let encoded = utf8_percent_encode(query, QUERY_ENCODE_SET).to_string();
        let route = format!("/search/issues?q={encoded}&per_page=1");
        let result: SearchResult = self.octocrab.get(route, None::<&()>).await?;
        Ok(result.total_count)
    }
}
