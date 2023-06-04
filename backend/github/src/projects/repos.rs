use axum::async_trait;
use reqwest::Client;
use shared::plugin::{ProjectInformation, ProjectsPlugin};

use crate::{api::rest, GithubConfig};

pub struct GithubProjectsRepos {
    config: GithubConfig,
    client: Client,
}
impl GithubProjectsRepos {
    pub fn new(config: &GithubConfig, client: &Client) -> Self {
        Self {
            config: config.clone(),
            client: client.clone(),
        }
    }
}

#[async_trait]
impl ProjectsPlugin for GithubProjectsRepos {
    async fn get_projects(&self, _username: &str, auth_token: &str) -> Vec<ProjectInformation> {
        rest::list_repositories(&self.config.rest_base, &self.client, auth_token)
            .await
            .unwrap()
            .iter()
            .map(|repo| repo.into())
            .collect()
    }
}
