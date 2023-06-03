use reqwest::{header, Client};
use serde::Deserialize;
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

pub async fn get_user(
    client: Client,
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
