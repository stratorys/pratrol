use tracing::error;

use super::constants::REQUEST_TIMEOUT;
use crate::config::Config;
use crate::domains::llm::error::LlmError;

pub struct MistralConnector {
    pub(crate) http_client: reqwest::Client,
    pub(crate) api_key: String,
    pub(crate) model: String,
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
