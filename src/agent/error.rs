use thiserror::Error;

use crate::domains::analysis::error::AnalysisError;
use crate::domains::llm::LlmError;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum GuardrailViolation {
    #[error("response echoed the untrusted-input boundary sentinel")]
    SentinelEcho,
    #[error("external URL present in {field}")]
    ExternalUrl { field: &'static str },
    #[error("recommends approval despite a high suspicious-pattern score")]
    InconsistentApproval,
}

#[derive(Debug, Error)]
pub enum DegradeReason {
    #[error("LLM unavailable: {0}")]
    LlmUnavailable(#[source] LlmError),
    #[error("unparsable response: {0}")]
    UnparsableResponse(#[source] AnalysisError),
    #[error("guardrail violations: {}", join_violations(.0))]
    GuardrailViolations(Vec<GuardrailViolation>),
}

fn join_violations(violations: &[GuardrailViolation]) -> String {
    violations
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}
