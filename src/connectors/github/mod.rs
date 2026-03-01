pub mod client;

use async_trait::async_trait;
use jsonwebtoken::EncodingKey;
use octocrab::Octocrab;

use crate::ports::github::{GitHubApp, GitHubError};

pub struct GitHubConnector {
    app: Octocrab,
}

impl GitHubConnector {
    pub fn new(app_id: u64, private_key: &str) -> Result<Self, GitHubError> {
        let key = EncodingKey::from_rsa_pem(private_key.as_bytes())?;
        let app = Octocrab::builder().app(app_id.into(), key).build()?;

        Ok(Self { app })
    }
}

pub struct InstalledClient {
    pub(crate) octocrab: Octocrab,
}

#[async_trait]
impl GitHubApp for GitHubConnector {
    type Client = InstalledClient;

    async fn installation_client(
        &self,
        installation_id: u64,
    ) -> Result<InstalledClient, GitHubError> {
        let (octocrab, _token) = self
            .app
            .installation_and_token(installation_id.into())
            .await?;

        Ok(InstalledClient { octocrab })
    }
}
