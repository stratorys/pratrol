pub mod chat;

use std::time::Duration;

use crate::config::Config;
use crate::domains::llm::LlmError;

pub struct GemmaConnector {
    http_client: reqwest::Client,
    base_url: String,
    model: String,
    api_key: Option<String>,
}

impl GemmaConnector {
    pub fn new(config: Config) -> Result<Self, LlmError> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;

        Ok(Self {
            http_client,
            base_url: config.gemma_base_url,
            model: config.gemma_model,
            api_key: config.gemma_api_key,
        })
    }
}
