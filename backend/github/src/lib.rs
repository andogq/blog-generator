mod api;
mod auth;
mod projects;
mod user;

use axum::http::{HeaderMap, HeaderValue};
use projects::repos::GithubProjectsRepos;
use reqwest::{
    header::{self, InvalidHeaderValue},
    Client, Url,
};

use shared::{
    environment::Environment,
    get_from_environment,
    plugin::{AuthPlugin, PluginCollection, PluginError, ProjectsPlugin, UserPlugin},
    source::Source,
};

use auth::oauth::GithubOAuth;
use user::GithubUserProfile;

pub struct Github {
    config: GithubConfig,
    client: Client,
}
impl Github {
    pub fn from_environment(environment: &Environment) -> Result<Self, PluginError> {
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
                    .ok_or(PluginError::MissingEnvVar("USER_AGENT".to_string()))?,
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
    rest_base: Url,
    oauth_base: Url,
}
impl GithubConfig {
    pub fn from_environment(environment: &Environment) -> Result<Self, PluginError> {
        Ok(Self {
            client_secret: get_from_environment!(environment, "GITHUB_CLIENT_SECRET"),
            client_id: get_from_environment!(environment, "GITHUB_CLIENT_ID"),
            rest_base: get_from_environment!(environment, "GITHUB_REST_BASE").parse()?,
            oauth_base: get_from_environment!(environment, "GITHUB_OAUTH_BASE").parse()?,
        })
    }
}

impl Source for Github {
    fn get_plugins(&self) -> PluginCollection {
        PluginCollection {
            auth: [(
                "oauth".to_string(),
                Box::new(GithubOAuth::new(&self.config, &self.client)) as Box<dyn AuthPlugin>,
            )]
            .into_iter()
            .collect(),
            user: [(
                "profile".to_string(),
                Box::new(GithubUserProfile::new(&self.config, &self.client)) as Box<dyn UserPlugin>,
            )]
            .into_iter()
            .collect(),
            project: [(
                "repos".to_string(),
                Box::new(GithubProjectsRepos::new(&self.config, &self.client))
                    as Box<dyn ProjectsPlugin>,
            )]
            .into_iter()
            .collect(),
        }
    }
}
