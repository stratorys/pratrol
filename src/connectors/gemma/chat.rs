use async_trait::async_trait;
use serde::{
    Deserialize,
    Serialize,
};

use super::GemmaConnector;
use crate::domains::llm::{
    ChatRequest,
    Llm,
    LlmError,
};

#[derive(Serialize)]
struct GemmaChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Deserialize)]
struct ChatChoiceMessage {
    content: String,
}

#[async_trait]
impl Llm for GemmaConnector {
    async fn chat_completion(
        &self,
        request: &ChatRequest,
    ) -> Result<String, LlmError> {
        let body = GemmaChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_owned(),
                    content: request.system.clone(),
                },
                ChatMessage {
                    role: "user".to_owned(),
                    content: request.user.clone(),
                },
            ],
        };

        let mut builder = self
            .http_client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&body);

        if let Some(api_key) = &self.api_key {
            builder = builder.header("Authorization", format!("Bearer {api_key}"));
        }

        let response = builder.send().await?.error_for_status()?;

        let chat_response: ChatResponse = response.json().await?;

        let content = chat_response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .ok_or(LlmError::EmptyResponse)?;

        Ok(content)
    }
}
