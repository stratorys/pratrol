use serde::Deserialize;
use tracing::warn;

use super::entity::AnalysisResult;
use super::error::AnalysisError;

pub fn response(raw: &str) -> Result<AnalysisResult, AnalysisError> {
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
    fn test_parse_response_valid_json() {
        let json = r#"{"code_coherence": 8.0, "commit_quality": 7.0, "risk_level": 2.0, "suspicious_patterns": 1.0, "summary": "A good PR."}"#;
        let result = response(json);
        assert!(result.is_ok(), "should parse valid JSON");
        let analysis = result.ok();
        assert!(analysis.is_some(), "result should be Some");
    }

    #[test]
    fn test_parse_response_invalid_json() {
        let result = response("not json at all");
        assert!(result.is_err(), "should fail on invalid JSON");
        assert!(
            matches!(result.err(), Some(AnalysisError::Json(_))),
            "should be Json error"
        );
    }

    #[test]
    fn test_parse_response_field_out_of_range() {
        let json = r#"{"code_coherence": 15.0, "commit_quality": 7.0, "risk_level": 2.0, "suspicious_patterns": 1.0, "summary": "test"}"#;
        let result = response(json);
        assert!(result.is_err(), "should fail on out-of-range field");
        assert!(
            matches!(result.err(), Some(AnalysisError::FieldOutOfRange)),
            "should be FieldOutOfRange error"
        );
    }

    #[test]
    fn test_parse_response_extracts_json_from_text() {
        let raw = r#"Here is my analysis:
{"code_coherence": 8.0, "commit_quality": 7.0, "risk_level": 2.0, "suspicious_patterns": 1.0, "summary": "test"}
Hope this helps!"#;
        let result = response(raw);
        assert!(result.is_ok(), "should extract JSON from surrounding text");
    }
}
