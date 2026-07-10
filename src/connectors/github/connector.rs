use std::sync::Arc;

use async_trait::async_trait;
use jsonwebtoken::EncodingKey;
use octocrab::Octocrab;
use tracing::error;

use super::constants::{
    CONNECT_TIMEOUT,
    READ_TIMEOUT,
};
use crate::domains::github::error::GitHubError;
use crate::domains::github::traits::{
    GitHubApp,
    GitHubClient,
};

pub struct GitHubConnector {
    app: Octocrab,
    bot_login: String,
}

impl GitHubConnector {
    pub async fn new(
        app_id: u64,
        private_key: &str,
    ) -> Result<Self, GitHubError> {
        let key = EncodingKey::from_rsa_pem(private_key.as_bytes()).map_err(|error| {
            error!(message = "Failed to parse GitHub App private key.", %error);
            GitHubError::Jwt
        })?;
        let app = Octocrab::builder()
            .set_connect_timeout(Some(CONNECT_TIMEOUT))
            .set_read_timeout(Some(READ_TIMEOUT))
            .app(app_id.into(), key)
            .build()
            .map_err(|error| {
                error!(message = "Failed to build GitHub App client.", %error, app_id);
                GitHubError::Api
            })?;

        let app_info: AppInfo = app.get("/app", None::<&()>).await.map_err(|error| {
            error!(message = "Failed to fetch GitHub App info.", %error, app_id);
            GitHubError::Api
        })?;
        let bot_login = format!("{}[bot]", app_info.slug);

        Ok(Self {
            app,
            bot_login,
        })
    }
}

#[derive(serde::Deserialize)]
struct AppInfo {
    slug: String,
}

pub struct InstalledClient {
    pub(crate) octocrab: Octocrab,
    pub(crate) bot_login: String,
}

#[async_trait]
impl GitHubApp for GitHubConnector {
    async fn installation_client(
        &self,
        installation_id: u64,
    ) -> Result<Arc<dyn GitHubClient>, GitHubError> {
        let (octocrab, _token) = self
            .app
            .installation_and_token(installation_id.into())
            .await
            .map_err(|error| {
                error!(
                    message = "Failed to create installation client.",
                    %error,
                    installation_id,
                );
                GitHubError::Api
            })?;

        Ok(Arc::new(InstalledClient {
            octocrab,
            bot_login: self.bot_login.clone(),
        }))
    }
}
