use reqwest::{header, Client};
use serde::Deserialize;
use shared::source::user::UserInformation;
use url::Url;

use crate::api::{rest::API_BASE, GithubApiError};

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

impl From<GetUserResponse> for UserInformation {
    fn from(user: GetUserResponse) -> Self {
        Self {
            name: user.name,
            avatar: user.avatar_url,
            bio: user.bio,
            location: user.location,
            email: user.email,
            links: [
                ("github".to_string(), Some(user.html_url)),
                (
                    "twitter".to_string(),
                    user.twitter_username
                        .map(|username| format!("https://twitter.com/{username}")),
                ),
            ]
            .into_iter()
            .filter_map(|(site, url)| url.map(|url| (site, url)))
            .collect(),
            blog: user.blog,
            company: user.company,
        }
    }
}

pub async fn get_user(
    client: &Client,
    access_token: &str,
) -> Result<GetUserResponse, GithubApiError> {
    let response = client
        .get(Url::parse(API_BASE).and_then(|url| url.join("user"))?)
        .header(header::AUTHORIZATION, format!("Bearer {access_token}"))
        .send()
        .await?;

    GithubApiError::match_status_code(response.status())?;

    response
        .json::<GetUserResponse>()
        .await
        .map_err(GithubApiError::Response)
}
