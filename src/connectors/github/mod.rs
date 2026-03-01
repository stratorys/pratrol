pub mod client;

use async_trait::async_trait;
use jsonwebtoken::EncodingKey;
use octocrab::Octocrab;

use crate::ports::github::{
    GitHubApp,
    GitHubError,
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
        let key = EncodingKey::from_rsa_pem(private_key.as_bytes())?;
        let app = Octocrab::builder().app(app_id.into(), key).build()?;

        let app_info: AppInfo = app.get("/app", None::<&()>).await?;
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
    type Client = InstalledClient;

    async fn installation_client(
        &self,
        installation_id: u64,
    ) -> Result<InstalledClient, GitHubError> {
        let (octocrab, _token) = self
            .app
            .installation_and_token(installation_id.into())
            .await?;

        Ok(InstalledClient {
            octocrab,
            bot_login: self.bot_login.clone(),
        })
    }
}
