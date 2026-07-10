use std::sync::Arc;

use super::entity::{
    AgentInput,
    AgentOutcome,
};
use super::error::DegradeReason;
use super::guardrails;
use crate::domains::analysis::{
    parse,
    prompt,
};
use crate::domains::llm::traits::Llm;

pub struct Harness {
    llm: Arc<dyn Llm>,
}

impl Harness {
    pub fn new(llm: Arc<dyn Llm>) -> Self {
        Self {
            llm,
        }
    }

    pub async fn run(
        &self,
        input: &AgentInput<'_>,
    ) -> AgentOutcome {
        let request = match prompt::build(input.diff, input.commit_messages) {
            Ok(request) => request,
            Err(error) => return AgentOutcome::Degraded(DegradeReason::PromptRender(error)),
        };

        let raw = match self.llm.chat_completion(&request).await {
            Ok(raw) => raw,
            Err(error) => return AgentOutcome::Degraded(DegradeReason::LlmUnavailable(error)),
        };

        let analysis = match parse::response(&raw) {
            Ok(analysis) => analysis,
            Err(error) => {
                return AgentOutcome::Degraded(DegradeReason::UnparsableResponse(error));
            }
        };

        let violations = guardrails::evaluate(&raw, &analysis);
        if violations.is_empty() {
            AgentOutcome::Validated(analysis)
        } else {
            AgentOutcome::Degraded(DegradeReason::GuardrailViolations(violations))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::error::GuardrailViolation;
    use crate::domains::analysis::error::AnalysisError;
    use crate::domains::analysis::prompt::SENTINEL_PREFIX;
    use crate::domains::llm::error::LlmError;
    use crate::domains::llm::traits::MockLlm;

    fn mock_ok(raw: &str) -> MockLlm {
        let response = raw.to_owned();
        let mut llm = MockLlm::new();
        llm.expect_chat_completion()
            .returning(move |_| Ok(response.clone()));
        llm
    }

    async fn run_capture(llm: MockLlm) -> AgentOutcome {
        let harness = Harness::new(Arc::new(llm));
        let commits = ["Initial commit"];
        let input = AgentInput {
            diff: "diff --git a/file.rs b/file.rs",
            commit_messages: &commits,
        };
        harness.run(&input).await
    }

    #[tokio::test]
    async fn test_valid_response_is_validated() {
        let raw = r#"{"code_coherence": 8.0, "commit_quality": 7.0, "risk_level": 2.0, "suspicious_patterns": 1.0, "summary": "A good PR.", "key_signal": "Clean code.", "recommendation": "Approve."}"#;
        let outcome = run_capture(mock_ok(raw)).await;
        assert!(
            matches!(outcome, AgentOutcome::Validated(_)),
            "clean response should validate"
        );
    }

    #[tokio::test]
    async fn test_transport_error_is_llm_unavailable() {
        let mut llm = MockLlm::new();
        llm.expect_chat_completion()
            .returning(|_| Err(LlmError::EmptyResponse));
        let outcome = run_capture(llm).await;
        assert!(
            matches!(
                outcome,
                AgentOutcome::Degraded(DegradeReason::LlmUnavailable(_))
            ),
            "transport error should degrade to LlmUnavailable"
        );
    }

    #[tokio::test]
    async fn test_garbage_is_unparsable() {
        let outcome = run_capture(mock_ok("not json at all")).await;
        assert!(
            matches!(
                outcome,
                AgentOutcome::Degraded(DegradeReason::UnparsableResponse(AnalysisError::Json))
            ),
            "garbage should degrade to UnparsableResponse::Json"
        );
    }

    #[tokio::test]
    async fn test_out_of_range_score_is_unparsable() {
        let raw = r#"{"code_coherence": 15.0, "commit_quality": 7.0, "risk_level": 2.0, "suspicious_patterns": 1.0, "summary": "test", "key_signal": "signal", "recommendation": "review"}"#;
        let outcome = run_capture(mock_ok(raw)).await;
        assert!(
            matches!(
                outcome,
                AgentOutcome::Degraded(DegradeReason::UnparsableResponse(
                    AnalysisError::FieldOutOfRange
                ))
            ),
            "out-of-range score should degrade to UnparsableResponse::FieldOutOfRange"
        );
    }

    #[tokio::test]
    async fn test_sentinel_echo_is_guardrail_violation() {
        let raw = format!(
            r#"{{"code_coherence": 8.0, "commit_quality": 7.0, "risk_level": 2.0, "suspicious_patterns": 1.0, "summary": "leaked {SENTINEL_PREFIX}0123456789ABCDEF", "key_signal": "x", "recommendation": "y"}}"#
        );
        let outcome = run_capture(mock_ok(&raw)).await;
        assert!(
            matches!(
                outcome,
                AgentOutcome::Degraded(DegradeReason::GuardrailViolations(ref violations))
                    if *violations == [GuardrailViolation::SentinelEcho]
            ),
            "sentinel echo should be a guardrail violation"
        );
    }

    #[tokio::test]
    async fn test_url_in_summary_is_guardrail_violation() {
        let raw = r#"{"code_coherence": 8.0, "commit_quality": 7.0, "risk_level": 2.0, "suspicious_patterns": 1.0, "summary": "See https://evil.example", "key_signal": "x", "recommendation": "y"}"#;
        let outcome = run_capture(mock_ok(raw)).await;
        assert!(
            matches!(
                outcome,
                AgentOutcome::Degraded(DegradeReason::GuardrailViolations(ref violations))
                    if *violations == [GuardrailViolation::ExternalUrl { field: "summary" }]
            ),
            "URL in summary should be a guardrail violation"
        );
    }

    #[tokio::test]
    async fn test_inconsistent_approval_is_guardrail_violation() {
        let raw = r#"{"code_coherence": 3.0, "commit_quality": 2.0, "risk_level": 8.0, "suspicious_patterns": 9.0, "summary": "risky", "key_signal": "x", "recommendation": "looks good, approve"}"#;
        let outcome = run_capture(mock_ok(raw)).await;
        assert!(
            matches!(
                outcome,
                AgentOutcome::Degraded(DegradeReason::GuardrailViolations(ref violations))
                    if *violations == [GuardrailViolation::InconsistentApproval]
            ),
            "approval despite high suspicion should be a guardrail violation"
        );
    }
}
