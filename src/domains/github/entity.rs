pub struct UserInfo {
    pub account_age_days: u32,
    pub public_repos: u32,
    pub followers: u32,
}

pub struct CommitInfo {
    pub message: String,
}

#[derive(Clone)]
pub struct RejectedPrInfo {
    pub number: u64,
    pub title: String,
    pub html_url: String,
}

pub struct RejectedPrSearchResult {
    pub total_count: u32,
    pub items: Vec<RejectedPrInfo>,
}
