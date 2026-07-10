use super::entity::CommentPayload;
use crate::sanitize::text::sanitize_plain_text;

const TRIAGE_TEMPLATE: &str = include_str!("templates/triage.md");
const PARTIAL_NOTE: &str = include_str!("templates/partial_note.md");

pub fn markdown(payload: &CommentPayload) -> String {
    let profile_badge = tier_badge(&payload.profile_tier_icon, &payload.profile_tier_label);
    let quality_badge = tier_badge(&payload.quality_tier_icon, &payload.quality_tier_label);
    let combined_badge = tier_badge(&payload.combined_tier_icon, &payload.combined_tier_label);

    let output = TRIAGE_TEMPLATE
        .replace(
            "{{profile_score}}",
            &format!("{:.0}", payload.profile_score),
        )
        .replace("{{profile_badge}}", profile_badge)
        .replace("{{profile_tier}}", &payload.profile_tier_label)
        .replace(
            "{{quality_score}}",
            &format!("{:.0}", payload.quality_score),
        )
        .replace("{{quality_badge}}", quality_badge)
        .replace("{{quality_tier}}", &payload.quality_tier_label)
        .replace(
            "{{combined_score}}",
            &format!("{:.0}", payload.combined_score),
        )
        .replace("{{combined_badge}}", combined_badge)
        .replace("{{combined_tier}}", &payload.combined_tier_label)
        .replace("{{summary}}", &sanitize_plain_text(&payload.summary))
        .replace("{{key_signal}}", &sanitize_plain_text(&payload.key_signal))
        .replace(
            "{{recommendation}}",
            &sanitize_plain_text(&payload.recommendation),
        );

    let history = if payload.history_section.is_empty() {
        String::new()
    } else {
        payload.history_section.clone()
    };
    let partial_note = if payload.analysis_partial {
        PARTIAL_NOTE.to_owned()
    } else {
        String::new()
    };

    format!("{output}{history}{partial_note}")
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
            history_section: String::new(),
        }
    }

    #[test]
    fn test_render_contains_scores_and_tiers() {
        let output = markdown(&sample_payload(false));
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
        let output = markdown(&sample_payload(true));
        assert!(
            output.contains("AI analysis was unavailable"),
            "should contain partial note"
        );
    }

    #[test]
    fn test_render_sanitizes_ai_text_fields() {
        let mut payload = sample_payload(false);
        payload.summary = "Ping @security-team <script>alert(1)</script>".to_owned();

        let output = markdown(&payload);
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

        let output = markdown(&payload);

        assert!(
            output.contains("…"),
            "long unicode summary should be truncated with ellipsis"
        );
    }
}
