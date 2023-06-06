use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub name: Option<String>,
    pub avatar: String,
    pub bio: Option<String>,
    pub location: Option<String>,

    pub email: Option<String>,

    pub links: HashMap<String, String>,
    pub blog: Option<String>,
    pub company: Option<String>,
}
