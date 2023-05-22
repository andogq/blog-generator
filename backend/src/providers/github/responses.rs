use crate::providers::{ExternalSite, UserInformation};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetUserResponse {
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
impl From<GetUserResponse> for UserInformation {
    fn from(response: GetUserResponse) -> Self {
        Self {
            name: response.name,
            avatar: response.avatar_url,
            bio: response.bio,
            location: response.location,
            email: response.email,
            links: [
                (ExternalSite::Github, Some(response.html_url)),
                (
                    ExternalSite::Twitter,
                    response
                        .twitter_username
                        .map(|username| format!("https://twitter.com/{username}")),
                ),
            ]
            .into_iter()
            .filter_map(|(site, url)| url.map(|url| (site, url)))
            .collect(),
            blog: response.blog,
            company: response.company,
        }
    }
}

#[derive(Deserialize)]
pub struct OAuthAccessTokenResponse {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
}
