use std::sync::Arc;

use axum::async_trait;
use shared::plugin::{ProjectInformation, ProjectsPlugin};

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
impl ProjectsPlugin for RepoTags {
    async fn get_projects(&self, _username: &str, auth_token: &str) -> Vec<ProjectInformation> {
        self.rest_api
            .search
            .repositories
            .by_topics(auth_token, &["portfolio".to_string()])
            .await
            .unwrap()
            .iter()
            .map(ProjectInformation::from)
            .collect()
    }
}
