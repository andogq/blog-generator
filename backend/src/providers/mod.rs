pub mod github;

use axum::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use self::github::GithubProviderError;

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ExternalSite {
    #[serde(rename = "github")]
    Github,

    #[serde(rename = "twitter")]
    Twitter,
}

#[derive(Debug, Serialize)]
pub struct UserInformation {
    name: Option<String>,
    avatar: String,
    bio: Option<String>,
    location: Option<String>,

    email: Option<String>,

    links: HashMap<ExternalSite, String>,
    blog: Option<String>,
    company: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RepoStats {
    stars: usize,
    forks: usize,
    watchers: usize,
    issues: usize,
}

#[derive(Debug, Serialize)]
pub struct Project {
    name: String,
    description: Option<String>,
    url: Option<String>,
    repo_url: Option<String>,
    repo_stats: Option<RepoStats>,
    tags: Vec<String>,
    languages: Option<Vec<String>>,
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Github error: {0}")]
    Github(#[from] GithubProviderError),
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn get_user(&self, user: &str) -> Result<Option<UserInformation>, ProviderError>;
    async fn get_projects(&self, user: &str) -> Result<Vec<Project>, ProviderError>;
    async fn oauth_callback(&mut self, code: &str) -> Result<(), ProviderError>;
    fn get_oauth_link(&self) -> String;
}
