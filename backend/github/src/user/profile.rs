use axum::async_trait;
use reqwest::Client;
use shared::source::{user::UserInformation, IdentifiableSource, UserSource};

use crate::{api::rest, GithubConfig};

pub struct GithubUserProfile {
    identifier: String,
    config: GithubConfig,
    client: Client,
}

impl GithubUserProfile {
    pub fn new(identifier: &str, config: &GithubConfig, client: &Client) -> Self {
        Self {
            identifier: identifier.to_string(),
            config: config.clone(),
            client: client.clone(),
        }
    }
}

impl IdentifiableSource for GithubUserProfile {
    fn get_identifier(&self) -> String {
        self.identifier.to_string()
    }
}

#[async_trait]
impl UserSource for GithubUserProfile {
    async fn get_user(&self, _username: &str, auth_token: &str) -> UserInformation {
        // TODO: Don't do this :(
        rest::get_user(&self.client, auth_token)
            .await
            .unwrap()
            .into()
    }
}
