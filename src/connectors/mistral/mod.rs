pub mod chat;

use std::time::Duration;

use crate::config::Config;
use crate::ports::mistral::MistralError;

pub struct MistralConnector {
    http_client: reqwest::Client,
    config: Config,
}

impl MistralConnector {
    pub fn new(config: Config) -> Result<Self, MistralError> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;

        Ok(Self {
            http_client,
            config,
        })
    }
}
