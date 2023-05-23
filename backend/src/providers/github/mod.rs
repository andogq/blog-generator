mod responses;

use std::collections::HashMap;

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
    #[error("unauthenticated user")]
    UnauthenticatedUser,
}
type Result<T> = std::result::Result<T, GithubProviderError>;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub struct GithubProvider {
    api_base: Url,
    client_id: String,
    client_secret: String,

    client: Client,

    access_tokens: HashMap<String, String>,
}

impl GithubProvider {
    pub fn new(api_base: &str, client_id: &str, client_secret: &str) -> Result<Self> {
        Ok(Self {
            api_base: Url::parse(api_base)?,
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),

            client: Client::builder()
                .default_headers(
                    [(header::ACCEPT, "application/vnd.github+json")]
                        .into_iter()
                        .map(|(header, value)| -> Result<(HeaderName, HeaderValue)> {
                            Ok((header, HeaderValue::from_str(value)?))
                        })
                        .collect::<Result<HeaderMap>>()?,
                )
                .user_agent(APP_USER_AGENT)
                .build()?,

            access_tokens: HashMap::new(),
        })
    }

    async fn get_user(&self, username: &str) -> Result<Option<UserInformation>> {
        let response = self
            .client
            .get(self.api_base.join(&format!("/users/{username}"))?)
            .header(
                header::AUTHORIZATION,
                self.access_tokens
                    .get(username)
                    .ok_or(GithubProviderError::UnauthenticatedUser)?,
            )
            .send()
            .await?;

        Ok(if response.status().is_success() {
            Some(response.json::<GetUserResponse>().await?.into())
        } else {
            None
        })
    }

    async fn oauth_callback(&mut self, code: &str) -> Result<()> {
        let request = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .json(&json!({
                "client_id": self.client_id,
                "client_secret": self.client_secret,
                "code": code
            }))
            .build()?;

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

            if response.status().is_success() {
                let user_info = response.json::<GetUserResponse>().await?;
                self.access_tokens.insert(user_info.login, access_token);

                Ok(())
            } else {
                Err(GithubProviderError::OAuth("unable to request user".into()))
            }
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

    async fn oauth_callback(&mut self, code: &str) -> std::result::Result<(), ProviderError> {
        Ok(self.oauth_callback(code).await?)
    }

    fn get_oauth_link(&self) -> String {
        Url::parse_with_params(
            "https://github.com/login/oauth/authorize",
            [
                ("scope", "read:user"),
                ("client_id", &self.client_id),
                ("redirect_uri", "http://localhost:3000/auth/github/callback"),
            ],
        )
        .unwrap()
        .to_string()
    }
}
