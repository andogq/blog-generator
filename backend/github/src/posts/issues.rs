use std::sync::Arc;

use axum::async_trait;
use shared::plugin::{DataPlugin, PluginError, PluginIdentifier, PostsResponse};

use crate::api::rest::RestApi;

pub struct PostsIssues {
    rest_api: Arc<RestApi>,
}

impl PostsIssues {
    pub fn new(rest_api: &Arc<RestApi>) -> Self {
        Self {
            rest_api: Arc::clone(rest_api),
        }
    }
}

#[async_trait]
impl DataPlugin for PostsIssues {
    type D = PostsResponse;

    async fn get_data(&self, username: &str, auth_token: &str) -> Result<Self::D, PluginError> {
        Ok(self
            .rest_api
            .search
            .issues
            .builder(auth_token)
            .open()
            .repo(&format!("{username}/{username}"))
            .label("post")
            .search()
            .await?
            .into_iter()
            .map(|issue| issue.into())
            .collect())
    }

    fn get_identifier(&self) -> PluginIdentifier {
        PluginIdentifier::new("issues")
    }
}
