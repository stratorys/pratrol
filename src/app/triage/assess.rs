use tracing::warn;

use super::load::Context;
use crate::agent::entity::{
    AgentInput,
    AgentOutcome,
};
use crate::agent::error::DegradeReason;
use crate::agent::harness::Harness;
use crate::domains::scoring::compute;
use crate::domains::scoring::entity::{
    QualitySignals,
    Score,
};

pub struct Assessment {
    pub quality_score: Score,
    pub summary: String,
    pub key_signal: String,
    pub recommendation: String,
    pub partial: bool,
}

pub async fn pull_request(
    harness: &Harness,
    context: &Context,
) -> Assessment {
    let commit_messages: Vec<&str> = context
        .commits
        .iter()
        .map(|commit| commit.message.as_str())
        .collect();

    match harness
        .run(&AgentInput {
            diff: &context.diff,
            commit_messages: &commit_messages,
        })
        .await
    {
        AgentOutcome::Validated(analysis) => {
            let quality_signals = QualitySignals {
                code_coherence: analysis.code_coherence,
                commit_quality: analysis.commit_quality,
                risk_level: analysis.risk_level,
                suspicious_patterns: analysis.suspicious_patterns,
            };

            Assessment {
                quality_score: compute::quality_score(&quality_signals),
                summary: analysis.summary,
                key_signal: analysis.key_signal,
                recommendation: analysis.recommendation,
                partial: false,
            }
        }
        AgentOutcome::Degraded(reason) => degraded(&reason),
    }
}

fn degraded(reason: &DegradeReason) -> Assessment {
    warn!(message = "Agent degraded, using fallback analysis.", %reason);

    let (summary, key_signal) = match reason {
        DegradeReason::GuardrailViolations(_) => (
            "AI analysis was rejected by safety guardrails.",
            "AI output failed guardrail checks.",
        ),
        DegradeReason::LlmUnavailable(_) | DegradeReason::UnparsableResponse(_) => (
            "Analysis was partial due to an error contacting the AI service.",
            "AI analysis unavailable.",
        ),
    };

    Assessment {
        quality_score: Score {
            value: 0.0,
        },
        summary: summary.to_owned(),
        key_signal: key_signal.to_owned(),
        recommendation: "Manual review recommended.".to_owned(),
        partial: true,
    }
}
