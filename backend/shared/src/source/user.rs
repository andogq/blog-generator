use axum::async_trait;
use serde::Serialize;
use std::collections::HashMap;

use super::IdentifiableSource;

#[derive(Debug, Serialize)]
pub struct UserInformation {
    name: Option<String>,
    avatar: String,
    bio: Option<String>,
    location: Option<String>,

    email: Option<String>,

    links: HashMap<String, String>,
    blog: Option<String>,
    company: Option<String>,
}

#[async_trait]
pub trait UserSource: IdentifiableSource {
    async fn get_user(&self, username: &str, auth_token: &str) -> UserInformation;
}
