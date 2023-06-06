use axum::async_trait;
use serde::Serialize;

use super::PluginError;

#[async_trait]
pub trait DataPlugin: Send + Sync {
    type D: Serialize;

    async fn get_data(&self, username: &str, auth_token: &str) -> Result<Self::D, PluginError>;
}
