use std::sync::Arc;

use axum::async_trait;
use shared::plugin::{BlurbResponse, DataPlugin, PluginError, PluginIdentifier};

use crate::api::rest::RestApi;

pub struct BlurbReadme {
    rest_api: Arc<RestApi>,
}

impl BlurbReadme {
    pub fn new(rest_api: &Arc<RestApi>) -> Self {
        Self {
            rest_api: Arc::clone(rest_api),
        }
    }
}

#[async_trait]
impl DataPlugin for BlurbReadme {
    type D = BlurbResponse;

    async fn get_data(&self, username: &str, auth_token: &str) -> Result<Self::D, PluginError> {
        Ok(self
            .rest_api
            .repositories
            .get_readme(auth_token, username, username, true)
            .await?
            .into())
    }

    fn get_identifier(&self) -> PluginIdentifier {
        PluginIdentifier::new("readme")
    }
}
