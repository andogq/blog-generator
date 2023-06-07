use std::sync::Arc;

use axum::async_trait;
use shared::plugin::{DataPlugin, PluginError, PluginIdentifier, UserResponse};

use crate::api::rest::RestApi;

pub struct GithubUserProfile {
    rest_api: Arc<RestApi>,
}

impl GithubUserProfile {
    pub fn new(rest_api: &Arc<RestApi>) -> Self {
        Self {
            rest_api: Arc::clone(rest_api),
        }
    }
}

#[async_trait]
impl DataPlugin for GithubUserProfile {
    type D = UserResponse;

    async fn get_data(&self, _username: &str, auth_token: &str) -> Result<Self::D, PluginError> {
        Ok(self.rest_api.user.get(auth_token).await?.into())
    }

    fn get_identifier(&self) -> PluginIdentifier {
        PluginIdentifier::new("profile")
    }
}
