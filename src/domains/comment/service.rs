use super::entity::CommentPayload;

const TRIAGE_TEMPLATE: &str = include_str!("templates/triage.md");
const PARTIAL_NOTE: &str = include_str!("templates/partial_note.md");

pub struct CommentService;

impl CommentService {
    pub fn new() -> Self { Self }

    pub fn render(
        &self,
        payload: &CommentPayload,
    ) -> String {
        let profile_badge = tier_badge(&payload.profile_tier_icon, &payload.profile_tier_label);
        let quality_badge = tier_badge(&payload.quality_tier_icon, &payload.quality_tier_label);
        let combined_badge = tier_badge(&payload.combined_tier_icon, &payload.combined_tier_label);

        let mut output = TRIAGE_TEMPLATE
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
            .replace("{{summary}}", &payload.summary)
            .replace("{{key_signal}}", &payload.key_signal)
            .replace("{{recommendation}}", &payload.recommendation);

        if !payload.history_section.is_empty() {
            output.push_str(&payload.history_section);
        }

        if payload.analysis_partial {
            output.push_str(PARTIAL_NOTE);
        }

        output
    }
}

fn tier_badge(
    icon: &str,
    label: &str,
) -> &'static str {
    match icon {
        "+" => "🟢",
        "~" => "🟡",
        "-" => "🔴",
        _ => match label.to_ascii_lowercase().as_str() {
            "high" => "🟢",
            "medium" => "🟡",
            "low" => "🔴",
            _ => "⚪",
        },
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
        let service = CommentService::new();
        let output = service.render(&sample_payload(false));
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
        let service = CommentService::new();
        let output = service.render(&sample_payload(true));
        assert!(
            output.contains("AI analysis was unavailable"),
            "should contain partial note"
        );
    }
}
