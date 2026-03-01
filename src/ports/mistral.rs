use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum MistralError {
    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("empty response")]
    EmptyResponse,
}

#[async_trait]
pub trait MistralPort: Send + Sync {
    async fn chat_completion(&self, prompt: &str) -> Result<String, MistralError>;
}
