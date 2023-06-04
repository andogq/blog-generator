use axum::async_trait;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct UserInformation {
    pub name: Option<String>,
    pub avatar: String,
    pub bio: Option<String>,
    pub location: Option<String>,

    pub email: Option<String>,

    pub links: HashMap<String, String>,
    pub blog: Option<String>,
    pub company: Option<String>,
}

#[async_trait]
pub trait UserPlugin: Send + Sync {
    async fn get_user(&self, username: &str, auth_token: &str) -> UserInformation;
}
