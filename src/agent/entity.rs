use std::fmt;

use crate::domains::analysis::entity::AnalysisResult;
use crate::domains::analysis::error::AnalysisError;
use crate::domains::llm::LlmError;

pub struct AgentInput<'req> {
    pub diff: &'req str,
    pub commit_messages: &'req [&'req str],
}

#[derive(Debug, Clone, PartialEq)]
pub enum GuardrailViolation {
    SentinelEcho,
    ExternalUrl { field: &'static str },
    InconsistentApproval,
}

impl fmt::Display for GuardrailViolation {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::SentinelEcho => {
                formatter.write_str("response echoed the untrusted-input boundary sentinel")
            }
            Self::ExternalUrl {
                field,
            } => write!(formatter, "external URL present in {field}"),
            Self::InconsistentApproval => {
                formatter.write_str("recommends approval despite a high suspicious-pattern score")
            }
        }
    }
}

#[derive(Debug)]
pub enum AgentOutcome {
    Validated(AnalysisResult),
    Degraded(DegradeReason),
}

#[derive(Debug)]
pub enum DegradeReason {
    LlmUnavailable(LlmError),
    UnparsableResponse(AnalysisError),
    GuardrailViolations(Vec<GuardrailViolation>),
}

impl fmt::Display for DegradeReason {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::LlmUnavailable(error) => write!(formatter, "LLM unavailable: {error}"),
            Self::UnparsableResponse(error) => write!(formatter, "unparsable response: {error}"),
            Self::GuardrailViolations(violations) => {
                write!(formatter, "guardrail violations: ")?;
                for (index, violation) in violations.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "{violation}")?;
                }
                Ok(())
            }
        }
    }
}
