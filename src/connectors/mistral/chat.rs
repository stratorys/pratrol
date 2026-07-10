use async_trait::async_trait;
use serde::{
    Deserialize,
    Serialize,
};

use super::MistralConnector;
use crate::domains::llm::{
    ChatRequest,
    Llm,
    LlmError,
};

#[derive(Serialize)]
struct MistralChatRequest {
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
impl Llm for MistralConnector {
    async fn chat_completion(
        &self,
        request: &ChatRequest,
    ) -> Result<String, LlmError> {
        let request = MistralChatRequest {
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

        let response = self
            .http_client
            .post("https://api.mistral.ai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

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
