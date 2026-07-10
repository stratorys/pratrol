use crate::domains::github::entity::RejectedPrInfo;

pub struct HistorySignals {
    pub author_in_repo: u32,
    pub author_in_repo_items: Vec<RejectedPrInfo>,
    pub title_in_repo: u32,
    pub title_in_repo_items: Vec<RejectedPrInfo>,
    pub author_global: u32,
}

pub struct HistoryResult {
    pub penalty: f64,
    pub is_repeat_offender: bool,
    pub presentation: HistoryPresentation,
}

#[derive(Clone)]
pub enum HistoryPresentation {
    Unavailable,
    Available(HistoryDetails),
}

#[derive(Clone)]
pub struct HistoryDetails {
    pub author_in_repo: u32,
    pub author_in_repo_items: Vec<RejectedPrInfo>,
    pub title_in_repo: u32,
    pub title_in_repo_items: Vec<RejectedPrInfo>,
    pub author_global: u32,
    pub is_repeat_offender: bool,
    pub current_pr_number: u64,
}
