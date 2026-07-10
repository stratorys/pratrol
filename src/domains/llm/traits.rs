use async_trait::async_trait;

pub use super::entity::ChatRequest;
pub use super::error::LlmError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Llm: Send + Sync {
    async fn chat_completion(
        &self,
        request: &ChatRequest,
    ) -> Result<String, LlmError>;
}
