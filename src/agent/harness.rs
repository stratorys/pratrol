use std::sync::Arc;

use super::entity::{
    AgentInput,
    AgentOutcome,
    DegradeReason,
};
use super::executor::Executor;
use super::guardrails;
use crate::app::analysis::service::AnalysisService;
use crate::domains::llm::Llm;

pub struct Harness {
    llm: Arc<dyn Llm>,
    analysis: AnalysisService,
}

impl Harness {
    pub fn new(llm: Arc<dyn Llm>) -> Self {
        Self {
            llm,
            analysis: AnalysisService::new(),
        }
    }

    pub async fn run<E: Executor>(
        &self,
        input: &AgentInput<'_>,
        executor: &E,
    ) -> Result<(), E::Error> {
        let prompt = self
            .analysis
            .build_prompt(input.diff, input.commit_messages);

        let outcome = match self.llm.chat_completion(&prompt).await {
            Err(error) => AgentOutcome::Degraded(DegradeReason::LlmUnavailable(error)),
            Ok(raw) => match self.analysis.parse_response(&raw) {
                Err(error) => AgentOutcome::Degraded(DegradeReason::UnparsableResponse(error)),
                Ok(analysis) => {
                    let violations = guardrails::evaluate(&raw, &analysis);
                    if violations.is_empty() {
                        AgentOutcome::Validated(analysis)
                    } else {
                        AgentOutcome::Degraded(DegradeReason::GuardrailViolations(violations))
                    }
                }
            },
        };

        executor.execute(outcome).await
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::sync::Mutex;

    use async_trait::async_trait;

    use super::*;
    use crate::agent::entity::GuardrailViolation;
    use crate::app::analysis::SENTINEL_PREFIX;
    use crate::domains::analysis::error::AnalysisError;
    use crate::domains::llm::{
        LlmError,
        MockLlm,
    };

    struct RecordingExecutor {
        captured: Mutex<Option<AgentOutcome>>,
    }

    #[async_trait]
    impl Executor for RecordingExecutor {
        type Error = Infallible;

        async fn execute(
            &self,
            outcome: AgentOutcome,
        ) -> Result<(), Self::Error> {
            let mut slot = self
                .captured
                .lock()
                .expect("capture lock should not be poisoned");
            *slot = Some(outcome);
            Ok(())
        }
    }

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
        let executor = RecordingExecutor {
            captured: Mutex::new(None),
        };

        let result = harness.run(&input, &executor).await;
        assert!(result.is_ok(), "run should complete");

        executor
            .captured
            .lock()
            .expect("capture lock should not be poisoned")
            .take()
            .expect("an outcome should be recorded")
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
                AgentOutcome::Degraded(DegradeReason::UnparsableResponse(AnalysisError::Json(_)))
            ),
            "garbage should degrade to UnparsableResponse::Json"
        );
    }

    #[tokio::test]
    async fn test_out_of_range_score_is_unparsable() {
        let raw = r#"{"code_coherence": 15.0, "commit_quality": 7.0, "risk_level": 2.0, "suspicious_patterns": 1.0, "summary": "test"}"#;
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
