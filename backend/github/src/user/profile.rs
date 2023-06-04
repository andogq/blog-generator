use std::sync::Arc;

use axum::async_trait;
use shared::plugin::{UserInformation, UserPlugin};

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
impl UserPlugin for GithubUserProfile {
    async fn get_user(&self, _username: &str, auth_token: &str) -> UserInformation {
        // TODO: Don't do this :(
        self.rest_api.user.get(auth_token).await.unwrap().into()
    }
}
