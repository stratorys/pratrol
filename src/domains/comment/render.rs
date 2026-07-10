use askama::Template;

use super::entity::CommentPayload;
use super::history;
use crate::sanitize::text::{
    SanitizedText,
    sanitize_plain_text,
};

#[derive(Template)]
#[template(path = "domains/comment/templates/triage.md")]
struct TriageTemplate<'payload> {
    combined_badge: &'payload str,
    combined_tier: &'payload str,
    profile_score: String,
    profile_badge: &'payload str,
    profile_tier: &'payload str,
    quality_score: String,
    quality_badge: &'payload str,
    quality_tier: &'payload str,
    combined_score: String,
    summary: SanitizedText,
    key_signal: SanitizedText,
    recommendation: SanitizedText,
    history: String,
    partial: bool,
}

pub fn markdown(payload: &CommentPayload) -> Result<String, askama::Error> {
    TriageTemplate {
        combined_badge: tier_badge(&payload.combined_tier_icon, &payload.combined_tier_label),
        combined_tier: &payload.combined_tier_label,
        profile_score: format!("{:.0}", payload.profile_score),
        profile_badge: tier_badge(&payload.profile_tier_icon, &payload.profile_tier_label),
        profile_tier: &payload.profile_tier_label,
        quality_score: format!("{:.0}", payload.quality_score),
        quality_badge: tier_badge(&payload.quality_tier_icon, &payload.quality_tier_label),
        quality_tier: &payload.quality_tier_label,
        combined_score: format!("{:.0}", payload.combined_score),
        summary: sanitize_plain_text(&payload.summary),
        key_signal: sanitize_plain_text(&payload.key_signal),
        recommendation: sanitize_plain_text(&payload.recommendation),
        history: history::render(&payload.history),
        partial: payload.analysis_partial,
    }
    .render()
}

fn tier_badge(
    icon: &str,
    label: &str,
) -> &'static str {
    let icon_badge = match icon {
        "+" => "🟢",
        "~" => "🟡",
        "-" => "🔴",
        _ => "",
    };

    if !icon_badge.is_empty() {
        return icon_badge;
    }

    match label.to_ascii_lowercase().as_str() {
        "high" => "🟢",
        "medium" => "🟡",
        "low" => "🔴",
        _ => "⚪",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::history::entity::{
        HistoryDetails,
        HistoryPresentation,
    };

    fn empty_history() -> HistoryPresentation {
        HistoryPresentation::Available(HistoryDetails {
            author_in_repo: 0,
            author_in_repo_items: vec![],
            title_in_repo: 0,
            title_in_repo_items: vec![],
            author_global: 0,
            is_repeat_offender: false,
            current_pr_number: 1,
        })
    }

    fn sample_payload(partial: bool) -> CommentPayload {
        CommentPayload {
            profile_score: 75.0,
            profile_tier_label: "High".to_owned(),
            profile_tier_icon: "+".to_owned(),
            quality_score: 60.0,
            quality_tier_label: "Medium".to_owned(),
            quality_tier_icon: "~".to_owned(),
            combined_score: 66.0,
            combined_tier_label: "Medium".to_owned(),
            combined_tier_icon: "~".to_owned(),
            summary: "A good PR.".to_owned(),
            key_signal: "Clean separation of concerns.".to_owned(),
            recommendation: "Approve after verifying tests pass.".to_owned(),
            analysis_partial: partial,
            history: empty_history(),
        }
    }

    fn render(payload: &CommentPayload) -> String {
        markdown(payload).expect("triage comment should render")
    }

    #[test]
    fn test_render_contains_scores_and_tiers() {
        let output = render(&sample_payload(false));
        assert!(
            output.contains("Pratrol Triage Brief"),
            "should contain brief heading"
        );
        assert!(
            output.contains("Risk Vector"),
            "should contain risk table header"
        );
        assert!(output.contains("75/100"), "should contain profile score");
        assert!(
            output.contains("🟢 **High**"),
            "should contain profile tier badge"
        );
        assert!(output.contains("60/100"), "should contain quality score");
        assert!(
            output.contains("🟡 **Medium**"),
            "should contain quality tier badge"
        );
        assert!(output.contains("66/100"), "should contain combined score");
        assert!(output.contains("A good PR."), "should contain summary");
        assert!(
            output.contains("Clean separation of concerns."),
            "should contain key signal"
        );
        assert!(
            output.contains("Approve after verifying tests pass."),
            "should contain recommendation"
        );
        assert!(
            !output.contains("AI analysis was unavailable"),
            "should not contain partial note"
        );
    }

    #[test]
    fn test_render_partial_appends_note() {
        let output = render(&sample_payload(true));
        assert!(
            output.contains("AI analysis was unavailable"),
            "should contain partial note"
        );
    }

    #[test]
    fn test_render_sanitizes_ai_text_fields() {
        let mut payload = sample_payload(false);
        payload.summary = "Ping @security-team <script>alert(1)</script>".to_owned();

        let output = render(&payload);
        assert!(
            output.contains("@\u{200B}security-team"),
            "should neutralize mentions"
        );
        assert!(
            !output.contains("<script>"),
            "should escape angle brackets: {output}"
        );
        assert!(
            output.contains("alert(1)"),
            "text content should remain: {output}"
        );
    }

    #[test]
    fn test_render_handles_unicode_without_panicking() {
        let mut payload = sample_payload(false);
        payload.summary = "é".repeat(301);

        let output = render(&payload);

        assert!(
            output.contains("…"),
            "long unicode summary should be truncated with ellipsis"
        );
    }
}
