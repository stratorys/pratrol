#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("llm request failed")]
    Http,

    #[error("empty response")]
    EmptyResponse,
}
