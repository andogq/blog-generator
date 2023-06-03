mod auth;
mod rest_api;

pub use auth::oauth::GithubOAuth;
use reqwest::Url;
use shared::{
    environment::{Environment},
    get_from_environment,
    source::{AuthSource, Source, SourceCollection, SourceError},
};

pub struct Github {
    config: GithubConfig,
}
impl Github {
    pub fn from_environment(environment: &Environment) -> Result<Self, SourceError> {
        Ok(Self {
            config: GithubConfig::from_environment(environment)?,
        })
    }
}

#[derive(Clone)]
pub struct GithubConfig {
    client_secret: String,
    client_id: String,
    api_base: Url,
    oauth_base: Url,
}
impl GithubConfig {
    pub fn from_environment(environment: &Environment) -> Result<Self, SourceError> {
        Ok(Self {
            client_secret: get_from_environment!(environment, "GITHUB_CLIENT_SECRET"),
            client_id: get_from_environment!(environment, "GITHUB_CLIENT_ID"),
            api_base: get_from_environment!(environment, "GITHUB_API_BASE").parse()?,
            oauth_base: get_from_environment!(environment, "GITHUB_OAUTH_BASE").parse()?,
        })
    }
}

impl Source for Github {
    fn get_sources(&self) -> SourceCollection {
        SourceCollection {
            auth: vec![Box::new(GithubOAuth::new(&self.config)) as Box<dyn AuthSource>],
            user: vec![],
            project: vec![],
        }
    }
}
