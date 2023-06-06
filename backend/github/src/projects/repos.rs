use std::sync::Arc;

use axum::async_trait;
use shared::plugin::{DataPlugin, PluginError, ProjectsResponse};

use crate::api::rest::RestApi;

pub struct GithubProjectsRepos {
    rest_api: Arc<RestApi>,
}
impl GithubProjectsRepos {
    pub fn new(rest_api: &Arc<RestApi>) -> Self {
        Self {
            rest_api: Arc::clone(rest_api),
        }
    }
}

#[async_trait]
impl DataPlugin for GithubProjectsRepos {
    type D = ProjectsResponse;

    async fn get_data(&self, _username: &str, auth_token: &str) -> Result<Self::D, PluginError> {
        Ok(self
            .rest_api
            .repositories
            .list(auth_token)
            .await?
            .iter()
            .map(|repo| repo.into())
            .collect())
    }
}
