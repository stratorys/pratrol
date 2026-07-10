use super::error::DegradeReason;
use crate::domains::analysis::entity::AnalysisResult;

pub struct AgentInput<'req> {
    pub diff: &'req str,
    pub commit_messages: &'req [&'req str],
}

#[derive(Debug)]
pub enum AgentOutcome {
    Validated(AnalysisResult),
    Degraded(DegradeReason),
}
