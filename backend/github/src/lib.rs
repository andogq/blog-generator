mod api;
mod auth;

pub use auth::oauth::GithubOAuth;
use shared::{
    environment::Environment,
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
            auth: vec![Box::new(GithubOAuth::new(&self.config)) as Box<dyn AuthSource>],
            user: vec![],
            project: vec![],
        }
    }
}
