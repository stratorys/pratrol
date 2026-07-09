use super::entity::GuardrailViolation;
use crate::app::analysis::SENTINEL_PREFIX;
use crate::domains::analysis::entity::AnalysisResult;

const SUSPICIOUS_APPROVAL_THRESHOLD: f64 = 8.0;

const URL_MARKERS: [&str; 3] = ["http://", "https://", "www."];

const APPROVAL_MARKERS: [&str; 4] = ["approve", "merge", "lgtm", "looks good"];

pub fn evaluate(
    raw: &str,
    analysis: &AnalysisResult,
) -> Vec<GuardrailViolation> {
    let mut violations = Vec::new();

    if raw.contains(SENTINEL_PREFIX) {
        violations.push(GuardrailViolation::SentinelEcho);
    }

    let fields = [
        ("summary", analysis.summary.as_str()),
        ("key_signal", analysis.key_signal.as_str()),
        ("recommendation", analysis.recommendation.as_str()),
    ];
    for (field, value) in fields {
        let lowered = value.to_lowercase();
        if URL_MARKERS.iter().any(|marker| lowered.contains(marker)) {
            violations.push(GuardrailViolation::ExternalUrl {
                field,
            });
        }
    }

    let recommendation = analysis.recommendation.to_lowercase();
    if analysis.suspicious_patterns >= SUSPICIOUS_APPROVAL_THRESHOLD
        && APPROVAL_MARKERS
            .iter()
            .any(|marker| recommendation.contains(marker))
    {
        violations.push(GuardrailViolation::InconsistentApproval);
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_analysis() -> AnalysisResult {
        AnalysisResult {
            code_coherence: 8.0,
            commit_quality: 7.0,
            risk_level: 2.0,
            suspicious_patterns: 1.0,
            summary: "A focused refactor.".to_owned(),
            key_signal: "Single module touched.".to_owned(),
            recommendation: "Standard review is enough.".to_owned(),
        }
    }

    #[test]
    fn test_clean_response_has_no_violations() {
        let analysis = clean_analysis();
        let violations = evaluate("clean raw response", &analysis);
        assert!(violations.is_empty(), "clean response should not violate");
    }

    #[test]
    fn test_g1_sentinel_echo() {
        let analysis = clean_analysis();
        let raw = format!("{SENTINEL_PREFIX}0123456789ABCDEF leaked");
        let violations = evaluate(&raw, &analysis);
        assert_eq!(
            violations,
            vec![GuardrailViolation::SentinelEcho],
            "sentinel echo should be the only violation"
        );
    }

    #[test]
    fn test_g2_external_url_in_summary() {
        let mut analysis = clean_analysis();
        analysis.summary = "See HTTPS://evil.example for details.".to_owned();
        let violations = evaluate("clean", &analysis);
        assert_eq!(
            violations,
            vec![GuardrailViolation::ExternalUrl {
                field: "summary"
            }],
            "uppercase URL in summary should violate case-insensitively"
        );
    }

    #[test]
    fn test_g2_external_url_in_key_signal_and_recommendation() {
        let mut analysis = clean_analysis();
        analysis.key_signal = "Visit www.evil.example now.".to_owned();
        analysis.recommendation = "Check http://evil.example first.".to_owned();
        let violations = evaluate("clean", &analysis);
        assert_eq!(
            violations,
            vec![
                GuardrailViolation::ExternalUrl {
                    field: "key_signal"
                },
                GuardrailViolation::ExternalUrl {
                    field: "recommendation"
                },
            ],
            "URLs in both fields should each violate"
        );
    }

    #[test]
    fn test_g3_inconsistent_approval() {
        let mut analysis = clean_analysis();
        analysis.suspicious_patterns = 9.0;
        analysis.recommendation = "This looks good, approve it.".to_owned();
        let violations = evaluate("clean", &analysis);
        assert_eq!(
            violations,
            vec![GuardrailViolation::InconsistentApproval],
            "high suspicion plus approval should violate"
        );
    }

    #[test]
    fn test_g3_high_suspicion_without_approval_is_clean() {
        let mut analysis = clean_analysis();
        analysis.suspicious_patterns = 9.0;
        analysis.recommendation = "Block and require manual review.".to_owned();
        let violations = evaluate("clean", &analysis);
        assert!(
            violations.is_empty(),
            "high suspicion alone should not violate"
        );
    }

    #[test]
    fn test_multiple_violations_collected_together() {
        let mut analysis = clean_analysis();
        analysis.suspicious_patterns = 9.0;
        analysis.summary = "Reach https://evil.example".to_owned();
        analysis.recommendation = "lgtm".to_owned();
        let raw = format!("{SENTINEL_PREFIX}DEADBEEFDEADBEEF");
        let violations = evaluate(&raw, &analysis);
        assert_eq!(
            violations,
            vec![
                GuardrailViolation::SentinelEcho,
                GuardrailViolation::ExternalUrl {
                    field: "summary"
                },
                GuardrailViolation::InconsistentApproval,
            ],
            "all violations should be collected in order"
        );
    }
}
