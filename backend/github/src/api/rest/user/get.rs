use reqwest::{header, Client};
use serde::Deserialize;
use shared::plugin::UserInformation;
use url::Url;

use crate::api::GithubApiError;

#[derive(Deserialize)]
pub struct GetUserResponse {
    pub login: String,
    pub avatar_url: String,
    pub html_url: String,
    pub name: Option<String>,
    pub company: Option<String>,
    pub blog: Option<String>,
    pub location: Option<String>,
    pub email: Option<String>,
    pub bio: Option<String>,
    pub twitter_username: Option<String>,
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
    api_base: &Url,
    client: &Client,
    access_token: &str,
) -> Result<GetUserResponse, GithubApiError> {
    let response = client
        .get(api_base.join("user")?)
        .header(header::AUTHORIZATION, format!("Bearer {access_token}"))
        .send()
        .await?;

    GithubApiError::match_status_code(response.status())?;

    response
        .json::<GetUserResponse>()
        .await
        .map_err(GithubApiError::Response)
}
