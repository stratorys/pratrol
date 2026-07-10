pub mod chat;

use std::time::Duration;

use crate::config::Config;
use crate::domains::llm::LlmError;

const REQUEST_TIMEOUT: Duration = Duration::from_mins(1);

pub struct GemmaConnector {
    http_client: reqwest::Client,
    base_url: String,
    model: String,
    api_key: Option<String>,
}

impl GemmaConnector {
    pub fn new(config: &Config) -> Result<Self, LlmError> {
        let http_client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()?;

        Ok(Self {
            http_client,
            base_url: config.gemma_base_url.clone(),
            model: config.gemma_model.clone(),
            api_key: config.gemma_api_key.clone(),
        })
    }
}
