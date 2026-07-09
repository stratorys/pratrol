use std::net::SocketAddr;
use std::{
    env,
    fs,
};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing environment variable: {name}")]
    MissingEnvVar { name: String },

    #[error("failed to read private key file at {path}: {source}")]
    PrivateKeyRead {
        path: String,
        source: std::io::Error,
    },

    #[error("invalid listen address: {value}: {source}")]
    InvalidListenAddr {
        value: String,
        source: std::net::AddrParseError,
    },

    #[error("invalid app id: {value}: {source}")]
    InvalidAppId {
        value: String,
        source: std::num::ParseIntError,
    },
}

#[derive(Clone)]
pub struct Config {
    pub github_app_id: u64,
    pub github_private_key: String,
    pub github_webhook_secret: String,
    pub listen_addr: SocketAddr,
    #[cfg(feature = "mistral")]
    pub mistral_api_key: String,
    #[cfg(feature = "mistral")]
    pub mistral_model: String,
    #[cfg(feature = "gemma")]
    pub gemma_base_url: String,
    #[cfg(feature = "gemma")]
    pub gemma_model: String,
    #[cfg(feature = "gemma")]
    pub gemma_api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let github_app_id_raw = require_env("GITHUB_APP_ID")?;
        let github_app_id =
            github_app_id_raw
                .parse::<u64>()
                .map_err(|source| ConfigError::InvalidAppId {
                    value: github_app_id_raw,
                    source,
                })?;

        let private_key_path = require_env("GITHUB_PRIVATE_KEY_PATH")?;
        let github_private_key = fs::read_to_string(&private_key_path).map_err(|source| {
            ConfigError::PrivateKeyRead {
                path: private_key_path,
                source,
            }
        })?;

        let github_webhook_secret = require_env("GITHUB_WEBHOOK_SECRET")?;

        #[cfg(feature = "mistral")]
        let mistral_api_key = require_env("MISTRAL_API_KEY")?;

        let listen_addr_raw = env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into());
        let listen_addr = listen_addr_raw.parse::<SocketAddr>().map_err(|source| {
            ConfigError::InvalidListenAddr {
                value: listen_addr_raw,
                source,
            }
        })?;

        #[cfg(feature = "mistral")]
        let mistral_model =
            env::var("MISTRAL_MODEL").unwrap_or_else(|_| "mistral-small-latest".into());

        #[cfg(feature = "gemma")]
        let gemma_base_url =
            env::var("GEMMA_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into());
        #[cfg(feature = "gemma")]
        let gemma_model = env::var("GEMMA_MODEL").unwrap_or_else(|_| "gemma-4-12b-it-qat".into());
        #[cfg(feature = "gemma")]
        let gemma_api_key = env::var("GEMMA_API_KEY").ok();

        Ok(Self {
            github_app_id,
            github_private_key,
            github_webhook_secret,
            listen_addr,
            #[cfg(feature = "mistral")]
            mistral_api_key,
            #[cfg(feature = "mistral")]
            mistral_model,
            #[cfg(feature = "gemma")]
            gemma_base_url,
            #[cfg(feature = "gemma")]
            gemma_model,
            #[cfg(feature = "gemma")]
            gemma_api_key,
        })
    }
}

fn require_env(name: &str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::MissingEnvVar {
        name: name.to_owned(),
    })
}
