use std::sync::Arc;

use axum::async_trait;
use shared::plugin::{
    DataPlugin, PluginError, PluginIdentifier, ProjectResponse, ProjectsResponse,
};

use crate::api::rest::RestApi;

pub struct RepoTags {
    rest_api: Arc<RestApi>,
}

impl RepoTags {
    pub fn new(rest_api: &Arc<RestApi>) -> Self {
        Self {
            rest_api: Arc::clone(rest_api),
        }
    }
}

#[async_trait]
impl DataPlugin for RepoTags {
    type D = ProjectsResponse;

    async fn get_data(&self, _username: &str, auth_token: &str) -> Result<Self::D, PluginError> {
        Ok(self
            .rest_api
            .search
            .repositories
            .by_topics(auth_token, &["portfolio".to_string()])
            .await?
            .iter()
            .map(ProjectResponse::from)
            .collect())
    }

    fn get_identifier(&self) -> PluginIdentifier {
        PluginIdentifier::new("repo_topics")
    }
}
