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
use serde_json::json;
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
    #[error("OAuth error: {0}")]
    OAuth(String),
}
type Result<T> = std::result::Result<T, GithubProviderError>;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub struct GithubProvider {
    api_base: Url,
    client_id: String,
    client_secret: String,

    client: Client,
}

impl GithubProvider {
    pub fn new(api_base: &str, client_id: &str, client_secret: &str) -> Result<Self> {
        Ok(Self {
            api_base: Url::parse(api_base)?,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),

            client: Client::builder()
                .default_headers(
                    [
                        (header::ACCEPT, "application/vnd.github+json"),
                        (header::AUTHORIZATION, client_secret),
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

    async fn oauth_callback(&self, code: &str) -> Result<String> {
        let mut request = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .json(&json!({
                "client_id": self.client_id,
                "client_secret": self.client_secret,
                "code": code
            }))
            .build()?;

        // Remove auth header since it's for a different origin
        request.headers_mut().remove(header::AUTHORIZATION);

        // Make the request
        let response = self.client.execute(request).await?;

        if response.status().is_success() {
            let access_token = response
                .json::<OAuthAccessTokenResponse>()
                .await?
                .access_token;

            // Find out the user that authenticated
            let response = self
                .client
                .get(self.api_base.join("/user")?)
                .header(header::AUTHORIZATION, format!("Bearer {access_token}"))
                .send()
                .await?;

            Ok(response.text().await?)
        } else {
            Err(GithubProviderError::OAuth(response.text().await?))
        }
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

    async fn oauth_callback(&self, code: &str) -> std::result::Result<String, ProviderError> {
        Ok(self.oauth_callback(code).await?)
    }
}
