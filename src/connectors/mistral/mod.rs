pub mod chat;

use std::time::Duration;

use crate::ports::mistral::MistralError;

pub struct MistralConnector {
    http_client: reqwest::Client,
    api_key: String,
}

impl MistralConnector {
    pub fn new(api_key: String) -> Result<Self, MistralError> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;

        Ok(Self {
            http_client,
            api_key,
        })
    }
}
