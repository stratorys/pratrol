use std::fmt::Write;

use rand::RngExt;
use serde::Deserialize;
use tracing::warn;

use super::entity::AnalysisResult;
use super::error::AnalysisError;

const REVIEW_PROMPT_TEMPLATE: &str = include_str!("templates/review_prompt.md");

pub struct AnalysisService;

impl AnalysisService {
    pub fn new() -> Self { Self }

    pub fn build_prompt(
        &self,
        diff: &str,
        commits: &[&str],
    ) -> String {
        let mut commits_text = String::with_capacity(commits.len() * 80);
        for (index, message) in commits.iter().enumerate() {
            let _ = writeln!(commits_text, "{}. {}", index + 1, message);
        }

        let sentinel_diff = random_sentinel();
        let sentinel_commits = random_sentinel();

        REVIEW_PROMPT_TEMPLATE
            .replace("{{sentinel_diff}}", &sentinel_diff)
            .replace("{{sentinel_commits}}", &sentinel_commits)
            .replace("{{diff}}", diff)
            .replace("{{commits}}", &commits_text)
    }

    pub fn parse_response(
        &self,
        raw: &str,
    ) -> Result<AnalysisResult, AnalysisError> {
        let trimmed = extract_json(raw);

        let parsed: RawAnalysis = serde_json::from_str(trimmed)?;

        validate_range("code_coherence", parsed.code_coherence)?;
        validate_range("commit_quality", parsed.commit_quality)?;
        validate_range("risk_level", parsed.risk_level)?;
        validate_range("suspicious_patterns", parsed.suspicious_patterns)?;

        Ok(AnalysisResult {
            code_coherence: parsed.code_coherence,
            commit_quality: parsed.commit_quality,
            risk_level: parsed.risk_level,
            suspicious_patterns: parsed.suspicious_patterns,
            summary: parsed.summary,
            key_signal: parsed.key_signal,
            recommendation: parsed.recommendation,
        })
    }
}

#[derive(Deserialize)]
struct RawAnalysis {
    code_coherence: f64,
    commit_quality: f64,
    risk_level: f64,
    suspicious_patterns: f64,
    summary: String,
    #[serde(default)]
    key_signal: String,
    #[serde(default)]
    recommendation: String,
}

fn validate_range(
    field: &str,
    value: f64,
) -> Result<(), AnalysisError> {
    if !(0.0..=10.0).contains(&value) {
        warn!(
            message = "Analysis field out of range.",
            %field,
            %value,
        );
        return Err(AnalysisError::FieldOutOfRange);
    }
    Ok(())
}

/// Generate a random boundary token that an attacker cannot predict.
fn random_sentinel() -> String {
    let token: u64 = rand::rng().random();
    format!("BOUNDARY_{token:016X}")
}

fn extract_json(raw: &str) -> &str {
    let trimmed = raw.trim();
    if let Some(start) = trimmed.find('{')
        && let Some(end) = trimmed.rfind('}')
    {
        return &trimmed[start..=end];
    }
    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt_contains_diff_and_commits() {
        let service = AnalysisService::new();
        let diff = "diff --git a/file.rs";
        let commits = vec!["Initial commit", "Fix bug"];
        let prompt = service.build_prompt(diff, &commits);
        assert!(
            prompt.contains("diff --git a/file.rs"),
            "prompt should contain diff"
        );
        assert!(
            prompt.contains("Initial commit"),
            "prompt should contain first commit"
        );
        assert!(
            prompt.contains("Fix bug"),
            "prompt should contain second commit"
        );
    }

    #[test]
    fn test_build_prompt_uses_random_sentinels() {
        let service = AnalysisService::new();
        let prompt = service.build_prompt("diff content", &["commit"]);
        assert!(
            prompt.contains("BOUNDARY_"),
            "prompt should contain random sentinel markers"
        );
        assert!(
            !prompt.contains("{{sentinel_diff}}"),
            "sentinel placeholders should be replaced"
        );
        assert!(
            !prompt.contains("{{sentinel_commits}}"),
            "sentinel placeholders should be replaced"
        );
    }

    #[test]
    fn test_build_prompt_sentinels_are_unique() {
        let service = AnalysisService::new();
        let prompt1 = service.build_prompt("diff", &["commit"]);
        let prompt2 = service.build_prompt("diff", &["commit"]);

        let extract_sentinel = |prompt: &str| -> String {
            prompt
                .lines()
                .find(|line| line.contains("BOUNDARY_") && line.contains("BEGIN UNTRUSTED DIFF"))
                .map(|line| line.to_owned())
                .unwrap_or_default()
        };

        assert_ne!(
            extract_sentinel(&prompt1),
            extract_sentinel(&prompt2),
            "consecutive prompts should have different sentinels"
        );
    }

    #[test]
    fn test_parse_response_valid_json() {
        let service = AnalysisService::new();
        let json = r#"{"code_coherence": 8.0, "commit_quality": 7.0, "risk_level": 2.0, "suspicious_patterns": 1.0, "summary": "A good PR."}"#;
        let result = service.parse_response(json);
        assert!(result.is_ok(), "should parse valid JSON");
        let analysis = result.ok();
        assert!(analysis.is_some(), "result should be Some");
    }

    #[test]
    fn test_parse_response_invalid_json() {
        let service = AnalysisService::new();
        let result = service.parse_response("not json at all");
        assert!(result.is_err(), "should fail on invalid JSON");
        assert!(
            matches!(result.err(), Some(AnalysisError::Json(_))),
            "should be Json error"
        );
    }

    #[test]
    fn test_parse_response_field_out_of_range() {
        let service = AnalysisService::new();
        let json = r#"{"code_coherence": 15.0, "commit_quality": 7.0, "risk_level": 2.0, "suspicious_patterns": 1.0, "summary": "test"}"#;
        let result = service.parse_response(json);
        assert!(result.is_err(), "should fail on out-of-range field");
        assert!(
            matches!(result.err(), Some(AnalysisError::FieldOutOfRange)),
            "should be FieldOutOfRange error"
        );
    }

    #[test]
    fn test_parse_response_extracts_json_from_text() {
        let service = AnalysisService::new();
        let raw = r#"Here is my analysis:
{"code_coherence": 8.0, "commit_quality": 7.0, "risk_level": 2.0, "suspicious_patterns": 1.0, "summary": "test"}
Hope this helps!"#;
        let result = service.parse_response(raw);
        assert!(result.is_ok(), "should extract JSON from surrounding text");
    }
}
