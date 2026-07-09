use async_trait::async_trait;

use super::entity::AgentOutcome;

#[async_trait]
pub trait Executor {
    type Error;

    async fn execute(
        &self,
        outcome: AgentOutcome,
    ) -> Result<(), Self::Error>;
}
