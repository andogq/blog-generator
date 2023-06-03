use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetUserResponse {
    pub login: String,
    avatar_url: String,
    html_url: String,
    name: Option<String>,
    company: Option<String>,
    blog: Option<String>,
    location: Option<String>,
    email: Option<String>,
    bio: Option<String>,
    twitter_username: Option<String>,
}
