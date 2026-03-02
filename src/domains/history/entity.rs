use crate::ports::github::RejectedPrInfo;

pub struct HistorySignals {
    pub rejected_by_author_in_repo: u32,
    pub rejected_by_author_in_repo_items: Vec<RejectedPrInfo>,
    pub rejected_by_title_in_repo: u32,
    pub rejected_by_title_in_repo_items: Vec<RejectedPrInfo>,
    pub rejected_by_author_global: u32,
}

pub struct HistoryResult {
    pub penalty: f64,
    pub is_repeat_offender: bool,
    pub history_section: String,
}
