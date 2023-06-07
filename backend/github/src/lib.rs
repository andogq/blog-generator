mod api;
mod auth;
mod projects;
mod user;

use std::sync::Arc;

use api::{oauth::OauthApi, rest::RestApi};
use axum::http::{HeaderMap, HeaderValue};
use projects::{repo_topics::RepoTags, repos::GithubProjectsRepos};
use reqwest::{
    header::{self, InvalidHeaderValue},
    Client, Url,
};

use shared::{
    environment::Environment,
    get_from_environment,
    plugin::{AuthPlugin, Plugin, SourceError, ToPlugin},
    source::{Source, SourceIdentifier},
};

use auth::oauth::GithubOAuth;
use user::GithubUserProfile;

pub struct Github {
    rest_api: Arc<RestApi>,
    oauth_api: Arc<OauthApi>,
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

        let rest_api = Arc::new(RestApi::new(&client, &config.rest_base));
        let oauth_api = Arc::new(OauthApi::new(
            &client,
            &config.oauth_base,
            &config.client_id,
            &config.client_secret,
        ));

        Ok(Self {
            rest_api,
            oauth_api,
        })
    }
}

#[derive(Clone)]
pub struct GithubConfig {
    client_secret: String,
    client_id: String,
    rest_base: Url,
    oauth_base: Url,
}
impl GithubConfig {
    pub fn from_environment(environment: &Environment) -> Result<Self, SourceError> {
        Ok(Self {
            client_secret: get_from_environment!(environment, "GITHUB_CLIENT_SECRET"),
            client_id: get_from_environment!(environment, "GITHUB_CLIENT_ID"),
            rest_base: get_from_environment!(environment, "GITHUB_REST_BASE").parse()?,
            oauth_base: get_from_environment!(environment, "GITHUB_OAUTH_BASE").parse()?,
        })
    }
}

impl Source for Github {
    fn get_identifier(&self) -> SourceIdentifier {
        SourceIdentifier::new("github")
    }

    fn get_plugins(&self) -> Vec<Plugin> {
        vec![
            GithubUserProfile::new(&self.rest_api).to_plugin(),
            GithubProjectsRepos::new(&self.rest_api).to_plugin(),
            RepoTags::new(&self.rest_api).to_plugin(),
        ]
    }

    fn get_auth_plugins(&self) -> Vec<Box<dyn AuthPlugin>> {
        vec![Box::new(GithubOAuth::new(&self.rest_api, &self.oauth_api)) as Box<dyn AuthPlugin>]
    }
}
