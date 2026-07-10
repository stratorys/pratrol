use tracing::error;

use super::constants::REQUEST_TIMEOUT;
use crate::config::Config;
use crate::domains::llm::error::LlmError;

pub struct GemmaConnector {
    pub(crate) http_client: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) model: String,
    pub(crate) api_key: Option<String>,
}

impl GemmaConnector {
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
            base_url: config.gemma_base_url.clone(),
            model: config.gemma_model.clone(),
            api_key: config.gemma_api_key.clone(),
        })
    }
}
