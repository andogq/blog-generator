mod responses;

use self::responses::*;
use super::{Provider, ProviderError, UserInformation};
use axum::{
    async_trait,
    http::{HeaderMap, HeaderName, HeaderValue},
};
use reqwest::{
    header::{self, InvalidHeaderName, InvalidHeaderValue},
    Client, Url,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GithubProviderError {
    #[error("parsing Github API base failed: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("value is not valid header name: {0}")]
    HeaderName(#[from] InvalidHeaderName),
    #[error("value is not valid header value: {0}")]
    HeaderValue(#[from] InvalidHeaderValue),
    #[error("reqwuest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}
type Result<T> = std::result::Result<T, GithubProviderError>;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub struct GithubProvider {
    api_base: Url,

    client: Client,
}

impl GithubProvider {
    pub fn new(api_base: &str, auth_token: &str) -> Result<Self> {
        Ok(Self {
            api_base: Url::parse(api_base)?,

            client: Client::builder()
                .default_headers(
                    [
                        (header::ACCEPT, "application/vnd.github+json"),
                        (header::AUTHORIZATION, auth_token),
                    ]
                    .into_iter()
                    .map(|(header, value)| -> Result<(HeaderName, HeaderValue)> {
                        Ok((header, HeaderValue::from_str(value)?))
                    })
                    .collect::<Result<HeaderMap>>()?,
                )
                .user_agent(APP_USER_AGENT)
                .build()?,
        })
    }

    async fn get_user(&self, username: &str) -> Result<Option<UserInformation>> {
        let response = self
            .client
            .get(self.api_base.join(&format!("/users/{username}"))?)
            .send()
            .await?;

        Ok(if response.status().is_success() {
            Some(response.json::<GetUserResponse>().await?.into())
        } else {
            None
        })
    }
}

#[async_trait]
impl Provider for GithubProvider {
    async fn get_user(
        &self,
        username: &str,
    ) -> std::result::Result<Option<UserInformation>, ProviderError> {
        Ok(self.get_user(username).await?)
    }
}
