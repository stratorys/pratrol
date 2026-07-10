pub mod chat;

use std::time::Duration;

use tracing::error;

use crate::config::Config;
use crate::domains::llm::LlmError;

const REQUEST_TIMEOUT: Duration = Duration::from_mins(1);

pub struct MistralConnector {
    http_client: reqwest::Client,
    api_key: String,
    model: String,
}

impl MistralConnector {
    pub fn new(config: &Config) -> Result<Self, LlmError> {
        let http_client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|error| {
                error!(message = "Failed to build HTTP client.", %error);
                LlmError::Http
            })?;

        Ok(Self {
            http_client,
            api_key: config.mistral_api_key.clone(),
            model: config.mistral_model.clone(),
        })
    }
}
