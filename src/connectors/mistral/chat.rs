use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::MistralConnector;
use crate::ports::mistral::{MistralError, MistralPort};

#[derive(Serialize)]
struct ChatRequest {
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
impl MistralPort for MistralConnector {
    async fn chat_completion(&self, prompt: &str) -> Result<String, MistralError> {
        let request = ChatRequest {
            model: self.config.mistral_model.to_owned(),
            messages: vec![ChatMessage {
                role: "user".to_owned(),
                content: prompt.to_owned(),
            }],
        };

        let response = self
            .http_client
            .post("https://api.mistral.ai/v1/chat/completions")
            .header(
                "Authorization",
                format!("Bearer {}", self.config.mistral_api_key),
            )
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
            .ok_or(MistralError::EmptyResponse)?;

        Ok(content)
    }
}
