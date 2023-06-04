use axum::async_trait;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Repo {
    pub url: String,
    pub stars: usize,
    pub forks: usize,
    pub watchers: usize,
    pub issues: usize,
}

#[derive(Debug, Serialize)]
pub struct ProjectInformation {
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub repo: Option<Repo>,
    pub tags: Vec<String>,
    pub languages: Option<Vec<String>>,
}

#[async_trait]
pub trait ProjectsPlugin: Send + Sync {
    async fn get_projects(&self, username: &str, auth_token: &str) -> Vec<ProjectInformation>;
}
