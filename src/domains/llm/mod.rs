pub mod entity;
pub mod error;

use async_trait::async_trait;
pub use entity::ChatRequest;
pub use error::LlmError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Llm: Send + Sync {
    async fn chat_completion(
        &self,
        request: &ChatRequest,
    ) -> Result<String, LlmError>;
}
