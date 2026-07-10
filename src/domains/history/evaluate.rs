use super::constants::{
    AUTHOR_GLOBAL_DIVISOR,
    AUTHOR_GLOBAL_PENALTY_MAX,
    AUTHOR_REPO_DIVISOR,
    AUTHOR_REPO_PENALTY_MAX,
    MIN_KEYWORD_LEN,
    REPEAT_OFFENDER_THRESHOLD,
    TARGET_KEYWORDS,
    TITLE_DIVISOR,
    TITLE_PENALTY_MAX,
};
use super::entity::{
    HistoryDetails,
    HistoryPresentation,
    HistoryResult,
    HistorySignals,
};

const STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has", "in", "is", "it", "of",
    "on", "or", "the", "to", "was", "with", "fix", "add", "update", "bump", "chore", "feat", "ci",
    "cd", "wip", "do", "not", "no", "this", "that",
];

pub fn history(
    signals: &HistorySignals,
    current_pr_number: u64,
) -> HistoryResult {
    let author_repo_penalty = (f64::from(signals.author_in_repo) / AUTHOR_REPO_DIVISOR).min(1.0)
        * AUTHOR_REPO_PENALTY_MAX;

    let title_penalty =
        (f64::from(signals.title_in_repo) / TITLE_DIVISOR).min(1.0) * TITLE_PENALTY_MAX;

    let global_penalty = (f64::from(signals.author_global) / AUTHOR_GLOBAL_DIVISOR).min(1.0)
        * AUTHOR_GLOBAL_PENALTY_MAX;

    let penalty = author_repo_penalty + title_penalty + global_penalty;
    let is_repeat_offender = signals.author_in_repo >= REPEAT_OFFENDER_THRESHOLD;

    HistoryResult {
        penalty,
        is_repeat_offender,
        presentation: HistoryPresentation::Available(HistoryDetails {
            author_in_repo: signals.author_in_repo,
            author_in_repo_items: signals.author_in_repo_items.clone(),
            title_in_repo: signals.title_in_repo,
            title_in_repo_items: signals.title_in_repo_items.clone(),
            author_global: signals.author_global,
            is_repeat_offender,
            current_pr_number,
        }),
    }
}

pub fn unavailable() -> HistoryResult {
    HistoryResult {
        penalty: 0.0,
        is_repeat_offender: false,
        presentation: HistoryPresentation::Unavailable,
    }
}

pub fn title_keywords(title: &str) -> Option<String> {
    let words: Vec<String> = title
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= MIN_KEYWORD_LEN)
        .map(str::to_ascii_lowercase)
        .filter(|w| !STOP_WORDS.contains(&w.as_str()))
        .take(TARGET_KEYWORDS)
        .collect();

    if words.is_empty() {
        return None;
    }

    Some(words.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_signals() -> HistorySignals {
        HistorySignals {
            author_in_repo: 0,
            author_in_repo_items: vec![],
            title_in_repo: 0,
            title_in_repo_items: vec![],
            author_global: 0,
        }
    }

    #[test]
    fn test_zero_history_produces_no_penalty() {
        let result = history(&empty_signals(), 1);
        assert!(
            (result.penalty - 0.0).abs() < f64::EPSILON,
            "penalty should be zero"
        );
        assert!(!result.is_repeat_offender, "should not be repeat offender");
    }

    #[test]
    fn test_unavailable_history_has_no_penalty() {
        let result = unavailable();
        assert!(
            (result.penalty - 0.0).abs() < f64::EPSILON,
            "penalty should be zero"
        );
        assert!(!result.is_repeat_offender, "should not be repeat offender");
        assert!(
            matches!(result.presentation, HistoryPresentation::Unavailable),
            "presentation should be unavailable"
        );
    }

    #[test]
    fn test_moderate_history_applies_penalty() {
        let signals = HistorySignals {
            author_in_repo: 2,
            author_in_repo_items: vec![],
            title_in_repo: 1,
            title_in_repo_items: vec![],
            author_global: 5,
        };
        let result = history(&signals, 1);

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
        let signals = HistorySignals {
            author_in_repo: 3,
            author_in_repo_items: vec![],
            title_in_repo: 0,
            title_in_repo_items: vec![],
            author_global: 0,
        };
        let result = history(&signals, 42);
        assert!(
            result.is_repeat_offender,
            "3 rejections should trigger repeat offender"
        );
    }

    #[test]
    fn test_penalty_caps_at_maximum() {
        let signals = HistorySignals {
            author_in_repo: 100,
            author_in_repo_items: vec![],
            title_in_repo: 100,
            title_in_repo_items: vec![],
            author_global: 500,
        };
        let result = history(&signals, 1);
        let max_penalty = 10.0 + 5.0 + 5.0;
        assert!(
            (result.penalty - max_penalty).abs() < f64::EPSILON,
            "penalty should cap at {max_penalty}, got {}",
            result.penalty
        );
    }

    #[test]
    fn test_extract_keywords_normal_title() {
        let result = title_keywords("Add user authentication middleware");
        assert_eq!(result, Some("user authentication middleware".to_owned()));
    }

    #[test]
    fn test_extract_keywords_with_stop_words() {
        let result = title_keywords("fix: update the broken login flow");
        assert_eq!(result, Some("broken login flow".to_owned()));
    }

    #[test]
    fn test_extract_keywords_too_short() {
        let result = title_keywords("fix a bug");
        assert_eq!(result, Some("bug".to_owned()));
    }

    #[test]
    fn test_extract_keywords_empty() {
        let result = title_keywords("fix: add a");
        assert_eq!(result, None);
    }
}
