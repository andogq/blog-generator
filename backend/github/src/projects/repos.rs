use std::sync::Arc;

use axum::async_trait;
use shared::plugin::{ProjectInformation, ProjectsPlugin};

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
impl ProjectsPlugin for GithubProjectsRepos {
    async fn get_projects(&self, _username: &str, auth_token: &str) -> Vec<ProjectInformation> {
        self.rest_api
            .repositories
            .list(auth_token)
            .await
            .unwrap()
            .iter()
            .map(|repo| repo.into())
            .collect()
    }
}
