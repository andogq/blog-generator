use axum::async_trait;
use reqwest::Client;
use shared::source::{user::UserInformation, UserSource};

use crate::{api::rest, GithubConfig};

pub struct GithubUserProfile {
    config: GithubConfig,
    client: Client,
}

impl GithubUserProfile {
    pub fn new(config: &GithubConfig, client: &Client) -> Self {
        Self {
            config: config.clone(),
            client: client.clone(),
        }
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
