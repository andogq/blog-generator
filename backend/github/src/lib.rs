mod api;
mod auth;
mod user;

use std::collections::HashMap;

pub use auth::oauth::GithubOAuth;
use axum::http::{HeaderMap, HeaderValue};
use reqwest::{
    header::{self, InvalidHeaderValue},
    Client,
};
use shared::{
    environment::Environment,
    get_from_environment,
    source::{AuthSource, Source, SourceCollection, SourceError, UserSource},
};
use user::GithubUserProfile;

pub struct Github {
    config: GithubConfig,
    client: Client,
}
impl Github {
    pub fn from_environment(environment: &Environment) -> Result<Self, SourceError> {
        let config = GithubConfig::from_environment(environment)?;

        let client = Client::builder()
            .default_headers(
                [(header::ACCEPT, "application/vnd.github+json")]
                    .into_iter()
                    .map(|(header, value)| Ok((header, HeaderValue::from_str(value)?)))
                    .collect::<Result<HeaderMap, InvalidHeaderValue>>()
                    .unwrap(),
            )
            .user_agent(
                environment
                    .get("USER_AGENT")
                    .ok_or(SourceError::MissingEnvVar("USER_AGENT".to_string()))?,
            )
            .build()
            .unwrap();

        Ok(Self { config, client })
    }
}

#[derive(Clone)]
pub struct GithubConfig {
    client_secret: String,
    client_id: String,
}
impl GithubConfig {
    pub fn from_environment(environment: &Environment) -> Result<Self, SourceError> {
        Ok(Self {
            client_secret: get_from_environment!(environment, "GITHUB_CLIENT_SECRET"),
            client_id: get_from_environment!(environment, "GITHUB_CLIENT_ID"),
        })
    }
}

impl Source for Github {
    fn get_sources(&self) -> SourceCollection {
        SourceCollection {
            auth: [(
                "oauth".to_string(),
                Box::new(GithubOAuth::new(&self.config, &self.client)) as Box<dyn AuthSource>,
            )]
            .into_iter()
            .collect(),
            user: [(
                "profile".to_string(),
                Box::new(GithubUserProfile::new(&self.config, &self.client)) as Box<dyn UserSource>,
            )]
            .into_iter()
            .collect(),
            project: Vec::new(),
        }
    }
}
