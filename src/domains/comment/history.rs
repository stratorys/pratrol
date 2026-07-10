use std::fmt::Write;

use tracing::warn;

use crate::domains::github::entity::RejectedPrInfo;
use crate::domains::history::entity::{
    HistoryDetails,
    HistoryPresentation,
};
use crate::sanitize::text::sanitize_markdown_text;
use crate::sanitize::url::sanitize_url;

const UNAVAILABLE_SECTION: &str = "\n\n---\n\n### Prior History\n\n> **\u{26a0}\u{fe0f} History \
                                   Unavailable:** GitHub search could not be reached, so the \
                                   author's past contributions were not factored into this \
                                   review.\n";

pub(super) fn render(presentation: &HistoryPresentation) -> String {
    match presentation {
        HistoryPresentation::Unavailable => UNAVAILABLE_SECTION.to_owned(),
        HistoryPresentation::Available(details) => render_available(details),
    }
}

fn render_available(details: &HistoryDetails) -> String {
    let has_author_repo = details.author_in_repo > 0;
    let has_title = details.title_in_repo > 0;
    let has_global = details.author_global > 0;

    if !has_author_repo && !has_title && !has_global {
        return String::new();
    }

    let mut section = String::with_capacity(512);
    section.push_str("\n\n---\n\n### Prior History\n\n");

    if details.is_repeat_offender {
        section.push_str(
            "> **\u{26a0}\u{fe0f} Repeat Offender:** This author has multiple \
             closed-without-merge PRs in this repository.\n\n",
        );
    }

    if has_author_repo {
        let _ = writeln!(
            section,
            "**{} closed-without-merge PR(s) by this author** in this repo:",
            details.author_in_repo
        );
        append_pr_list(
            &mut section,
            &details.author_in_repo_items,
            details.current_pr_number,
        );
    }

    if has_title {
        let _ = writeln!(
            section,
            "**{} similar PR(s)** (by title) in this repo:",
            details.title_in_repo
        );
        append_pr_list(
            &mut section,
            &details.title_in_repo_items,
            details.current_pr_number,
        );
    }

    if has_global {
        let _ = writeln!(
            section,
            "**{} closed-without-merge PR(s) by this author** across all repos.",
            details.author_global
        );
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
        if let Some(safe_url) = sanitize_url(&item.html_url) {
            let _ = writeln!(output, "- [#{}]({safe_url}) — {safe_title}", item.number);
        } else {
            warn!(
                message = "Dropped non-HTTPS URL from history section.",
                pr_number = item.number,
            );
            let _ = writeln!(output, "- #{} — {safe_title}", item.number);
        }
    }
    output.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    fn details(items: Vec<RejectedPrInfo>) -> HistoryDetails {
        HistoryDetails {
            author_in_repo: u32::try_from(items.len()).unwrap_or(u32::MAX),
            author_in_repo_items: items,
            title_in_repo: 0,
            title_in_repo_items: vec![],
            author_global: 0,
            is_repeat_offender: false,
            current_pr_number: 1,
        }
    }

    #[test]
    fn test_zero_history_renders_empty_section() {
        let section = render(&HistoryPresentation::Available(details(vec![])));
        assert!(
            section.is_empty(),
            "no history should render an empty section"
        );
    }

    #[test]
    fn test_unavailable_notes_it() {
        let section = render(&HistoryPresentation::Unavailable);
        assert!(
            section.contains("History Unavailable"),
            "section should state history is unavailable: {section}"
        );
    }

    #[test]
    fn test_repeat_offender_lists_prs() {
        let mut view = details(vec![
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
        ]);
        view.author_in_repo = 3;
        view.is_repeat_offender = true;
        view.current_pr_number = 42;

        let section = render(&HistoryPresentation::Available(view));
        assert!(
            section.contains("Repeat Offender"),
            "should contain repeat offender warning"
        );
        assert!(section.contains("#10"), "should list rejected PR #10");
    }

    #[test]
    fn test_current_pr_excluded_from_list() {
        let mut view = details(vec![RejectedPrInfo {
            number: 42,
            title: "Same PR".to_owned(),
            html_url: "https://github.com/o/r/pull/42".to_owned(),
        }]);
        view.current_pr_number = 42;
        let section = render(&HistoryPresentation::Available(view));
        assert!(
            !section.contains("[#42]"),
            "current PR should be excluded from the list"
        );
    }

    #[test]
    fn test_section_sanitizes_title_and_mentions() {
        let view = details(vec![RejectedPrInfo {
            number: 41,
            title: "Ping @security-team [x](y) <script>alert(1)</script>".to_owned(),
            html_url: "https://github.com/o/r/pull/41".to_owned(),
        }]);
        let section = render(&HistoryPresentation::Available(view));
        assert!(
            section.contains("@\u{200B}security-team"),
            "mentions in history titles should be neutralized: {section}"
        );
        assert!(
            !section.contains("<script>"),
            "script tags should be stripped by ammonia: {section}"
        );
        assert!(
            !section.contains("javascript:"),
            "javascript URIs should not survive ammonia: {section}"
        );
    }

    #[test]
    fn test_section_rejects_non_https_url() {
        let view = details(vec![RejectedPrInfo {
            number: 41,
            title: "Some PR".to_owned(),
            html_url: "javascript:alert(1)".to_owned(),
        }]);
        let section = render(&HistoryPresentation::Available(view));
        assert!(
            !section.contains("javascript:"),
            "non-https URL should be dropped: {section}"
        );
        assert!(
            section.contains("#41"),
            "PR number should still appear without link: {section}"
        );
    }
}
