use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WebhookEvent {
    pub action: String,
    pub installation: Installation,
    pub repository: Repository,
    pub pull_request: PullRequestPayload,
}

#[derive(Debug, Deserialize)]
pub struct Installation {
    pub id: u64,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestPayload {
    pub number: u64,
    pub user: PullRequestUser,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestUser {
    pub login: String,
}
