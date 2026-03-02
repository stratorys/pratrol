use tracing::warn;

use super::entity::{
    HistoryResult,
    HistorySignals,
};
use crate::ports::github::RejectedPrInfo;
use crate::sanitize::{
    sanitize_markdown_text,
    sanitize_url,
};

const AUTHOR_REPO_PENALTY_MAX: f64 = 10.0;
const AUTHOR_REPO_DIVISOR: f64 = 5.0;
const TITLE_PENALTY_MAX: f64 = 5.0;
const TITLE_DIVISOR: f64 = 3.0;
const AUTHOR_GLOBAL_PENALTY_MAX: f64 = 5.0;
const AUTHOR_GLOBAL_DIVISOR: f64 = 20.0;

const REPEAT_OFFENDER_THRESHOLD: u32 = 3;

const STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has", "in", "is", "it", "of",
    "on", "or", "the", "to", "was", "with", "fix", "add", "update", "bump", "chore", "feat", "ci",
    "cd", "wip", "do", "not", "no", "this", "that",
];

const MIN_KEYWORD_LEN: usize = 3;
const TARGET_KEYWORDS: usize = 3;

pub struct HistoryService;

impl HistoryService {
    pub fn new() -> Self { Self }

    pub fn evaluate(
        &self,
        signals: &HistorySignals,
        current_pr_number: u64,
    ) -> HistoryResult {
        let author_repo_penalty =
            (f64::from(signals.rejected_by_author_in_repo) / AUTHOR_REPO_DIVISOR).min(1.0)
                * AUTHOR_REPO_PENALTY_MAX;

        let title_penalty = (f64::from(signals.rejected_by_title_in_repo) / TITLE_DIVISOR).min(1.0)
            * TITLE_PENALTY_MAX;

        let global_penalty = (f64::from(signals.rejected_by_author_global) / AUTHOR_GLOBAL_DIVISOR)
            .min(1.0)
            * AUTHOR_GLOBAL_PENALTY_MAX;

        let penalty = author_repo_penalty + title_penalty + global_penalty;
        let is_repeat_offender = signals.rejected_by_author_in_repo >= REPEAT_OFFENDER_THRESHOLD;

        let history_section =
            render_history_section(signals, current_pr_number, is_repeat_offender);

        HistoryResult {
            penalty,
            is_repeat_offender,
            history_section,
        }
    }
}

pub fn extract_title_keywords(title: &str) -> Option<String> {
    let words: Vec<String> = title
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= MIN_KEYWORD_LEN)
        .map(|w| w.to_ascii_lowercase())
        .filter(|w| !STOP_WORDS.contains(&w.as_str()))
        .take(TARGET_KEYWORDS)
        .collect();

    if words.is_empty() {
        return None;
    }

    Some(words.join(" "))
}

fn render_history_section(
    signals: &HistorySignals,
    current_pr_number: u64,
    is_repeat_offender: bool,
) -> String {
    let has_author_repo = signals.rejected_by_author_in_repo > 0;
    let has_title = signals.rejected_by_title_in_repo > 0;
    let has_global = signals.rejected_by_author_global > 0;

    if !has_author_repo && !has_title && !has_global {
        return String::new();
    }

    let mut section = String::with_capacity(512);
    section.push_str("\n\n---\n\n### Prior History\n\n");

    if is_repeat_offender {
        section.push_str(
            "> **\u{26a0}\u{fe0f} Repeat Offender:** This author has multiple \
             closed-without-merge PRs in this repository.\n\n",
        );
    }

    if has_author_repo {
        section.push_str(&format!(
            "**{} closed-without-merge PR(s) by this author** in this repo:\n",
            signals.rejected_by_author_in_repo
        ));
        append_pr_list(
            &mut section,
            &signals.rejected_by_author_in_repo_items,
            current_pr_number,
        );
    }

    if has_title {
        section.push_str(&format!(
            "**{} similar PR(s)** (by title) in this repo:\n",
            signals.rejected_by_title_in_repo
        ));
        append_pr_list(
            &mut section,
            &signals.rejected_by_title_in_repo_items,
            current_pr_number,
        );
    }

    if has_global {
        section.push_str(&format!(
            "**{} closed-without-merge PR(s) by this author** across all repos.\n",
            signals.rejected_by_author_global
        ));
    }

    section
}

fn append_pr_list(
    output: &mut String,
    items: &[RejectedPrInfo],
    current_pr_number: u64,
) {
    for item in items {
        if item.number == current_pr_number {
            continue;
        }
        let safe_title = sanitize_markdown_text(&item.title);
        match sanitize_url(&item.html_url) {
            Some(safe_url) => {
                output.push_str(&format!(
                    "- [#{}]({safe_url}) — {safe_title}\n",
                    item.number,
                ));
            }
            None => {
                warn!(
                    message = "Dropped non-HTTPS URL from history section.",
                    pr_number = item.number,
                );
                output.push_str(&format!("- #{} — {safe_title}\n", item.number));
            }
        }
    }
    output.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_signals() -> HistorySignals {
        HistorySignals {
            rejected_by_author_in_repo: 0,
            rejected_by_author_in_repo_items: vec![],
            rejected_by_title_in_repo: 0,
            rejected_by_title_in_repo_items: vec![],
            rejected_by_author_global: 0,
        }
    }

    #[test]
    fn test_zero_history_produces_no_penalty() {
        let service = HistoryService::new();
        let result = service.evaluate(&empty_signals(), 1);
        assert!(
            (result.penalty - 0.0).abs() < f64::EPSILON,
            "penalty should be zero"
        );
        assert!(!result.is_repeat_offender, "should not be repeat offender");
        assert!(
            result.history_section.is_empty(),
            "history section should be empty"
        );
    }

    #[test]
    fn test_moderate_history_applies_penalty() {
        let service = HistoryService::new();
        let signals = HistorySignals {
            rejected_by_author_in_repo: 2,
            rejected_by_author_in_repo_items: vec![],
            rejected_by_title_in_repo: 1,
            rejected_by_title_in_repo_items: vec![],
            rejected_by_author_global: 5,
        };
        let result = service.evaluate(&signals, 1);

        let expected_author_repo = (2.0 / 5.0) * 10.0;
        let expected_title = (1.0 / 3.0) * 5.0;
        let expected_global = (5.0 / 20.0) * 5.0;
        let expected = expected_author_repo + expected_title + expected_global;

        assert!(
            (result.penalty - expected).abs() < 0.01,
            "penalty should match expected: got {} expected {}",
            result.penalty,
            expected
        );
        assert!(
            !result.is_repeat_offender,
            "2 rejections should not trigger repeat offender"
        );
    }

    #[test]
    fn test_repeat_offender_threshold() {
        let service = HistoryService::new();
        let signals = HistorySignals {
            rejected_by_author_in_repo: 3,
            rejected_by_author_in_repo_items: vec![
                RejectedPrInfo {
                    number: 10,
                    title: "Bad PR 1".to_owned(),
                    html_url: "https://github.com/o/r/pull/10".to_owned(),
                },
                RejectedPrInfo {
                    number: 20,
                    title: "Bad PR 2".to_owned(),
                    html_url: "https://github.com/o/r/pull/20".to_owned(),
                },
            ],
            rejected_by_title_in_repo: 0,
            rejected_by_title_in_repo_items: vec![],
            rejected_by_author_global: 0,
        };
        let result = service.evaluate(&signals, 42);

        assert!(
            result.is_repeat_offender,
            "3 rejections should trigger repeat offender"
        );
        assert!(
            result.history_section.contains("Repeat Offender"),
            "should contain repeat offender warning"
        );
        assert!(
            result.history_section.contains("#10"),
            "should list rejected PR #10"
        );
    }

    #[test]
    fn test_penalty_caps_at_maximum() {
        let service = HistoryService::new();
        let signals = HistorySignals {
            rejected_by_author_in_repo: 100,
            rejected_by_author_in_repo_items: vec![],
            rejected_by_title_in_repo: 100,
            rejected_by_title_in_repo_items: vec![],
            rejected_by_author_global: 500,
        };
        let result = service.evaluate(&signals, 1);
        let max_penalty = 10.0 + 5.0 + 5.0;
        assert!(
            (result.penalty - max_penalty).abs() < f64::EPSILON,
            "penalty should cap at {max_penalty}, got {}",
            result.penalty
        );
    }

    #[test]
    fn test_extract_keywords_normal_title() {
        let result = extract_title_keywords("Add user authentication middleware");
        assert_eq!(result, Some("user authentication middleware".to_owned()));
    }

    #[test]
    fn test_extract_keywords_with_stop_words() {
        let result = extract_title_keywords("fix: update the broken login flow");
        assert_eq!(result, Some("broken login flow".to_owned()));
    }

    #[test]
    fn test_extract_keywords_too_short() {
        let result = extract_title_keywords("fix a bug");
        assert_eq!(result, Some("bug".to_owned()));
    }

    #[test]
    fn test_extract_keywords_empty() {
        let result = extract_title_keywords("fix: add a");
        assert_eq!(result, None);
    }

    #[test]
    fn test_current_pr_excluded_from_list() {
        let service = HistoryService::new();
        let signals = HistorySignals {
            rejected_by_author_in_repo: 1,
            rejected_by_author_in_repo_items: vec![RejectedPrInfo {
                number: 42,
                title: "Same PR".to_owned(),
                html_url: "https://github.com/o/r/pull/42".to_owned(),
            }],
            rejected_by_title_in_repo: 0,
            rejected_by_title_in_repo_items: vec![],
            rejected_by_author_global: 0,
        };
        let result = service.evaluate(&signals, 42);
        assert!(
            !result.history_section.contains("[#42]"),
            "current PR should be excluded from the list"
        );
    }

    #[test]
    fn test_history_section_sanitizes_title_and_mentions() {
        let service = HistoryService::new();
        let signals = HistorySignals {
            rejected_by_author_in_repo: 1,
            rejected_by_author_in_repo_items: vec![RejectedPrInfo {
                number: 41,
                title: "Ping @security-team [x](y) <script>alert(1)</script>".to_owned(),
                html_url: "https://github.com/o/r/pull/41".to_owned(),
            }],
            rejected_by_title_in_repo: 0,
            rejected_by_title_in_repo_items: vec![],
            rejected_by_author_global: 0,
        };
        let result = service.evaluate(&signals, 42);
        assert!(
            result.history_section.contains("@\u{200B}security-team"),
            "mentions in history titles should be neutralized: {}",
            result.history_section,
        );
        assert!(
            !result.history_section.contains("<script>"),
            "script tags should be stripped by ammonia: {}",
            result.history_section,
        );
        assert!(
            !result.history_section.contains("javascript:"),
            "javascript URIs should not survive ammonia: {}",
            result.history_section,
        );
    }

    #[test]
    fn test_history_section_rejects_non_https_url() {
        let service = HistoryService::new();
        let signals = HistorySignals {
            rejected_by_author_in_repo: 1,
            rejected_by_author_in_repo_items: vec![RejectedPrInfo {
                number: 41,
                title: "Some PR".to_owned(),
                html_url: "javascript:alert(1)".to_owned(),
            }],
            rejected_by_title_in_repo: 0,
            rejected_by_title_in_repo_items: vec![],
            rejected_by_author_global: 0,
        };
        let result = service.evaluate(&signals, 42);
        assert!(
            !result.history_section.contains("javascript:"),
            "non-https URL should be dropped: {}",
            result.history_section,
        );
        assert!(
            result.history_section.contains("#41"),
            "PR number should still appear without link: {}",
            result.history_section,
        );
    }
}
