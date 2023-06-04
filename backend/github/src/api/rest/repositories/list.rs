use reqwest::{header, Client, Url};
use serde::Deserialize;
use shared::plugin::{ProjectInformation, Repo};

use crate::{
    api::{rest::API_BASE, GithubApiError},
};

#[derive(Deserialize)]
pub struct RepositoryResponse {
    pub name: String,
    pub private: bool,
    pub html_url: String,
    pub description: Option<String>,
    pub forks_count: usize,
    pub stargazers_count: usize,
    pub watchers_count: usize,
    pub open_issues_count: usize,
    pub topics: Vec<String>,
    pub homepage: Option<String>,
    pub language: Option<String>,
}

impl From<&RepositoryResponse> for Repo {
    fn from(repository: &RepositoryResponse) -> Self {
        Self {
            url: repository.html_url.clone(),
            stars: repository.stargazers_count,
            forks: repository.forks_count,
            watchers: repository.watchers_count,
            issues: repository.open_issues_count,
        }
    }
}

impl From<&RepositoryResponse> for ProjectInformation {
    fn from(repository: &RepositoryResponse) -> Self {
        Self {
            name: repository.name.clone(),
            description: repository.description.clone(),
            url: repository.homepage.clone(),
            repo: (!repository.private).then_some(Repo::from(repository)),
            tags: repository.topics.clone(),
            languages: repository.language.clone().map(|language| vec![language]),
        }
    }
}

pub async fn list_repositories(
    client: &Client,
    access_token: &str,
) -> Result<Vec<RepositoryResponse>, GithubApiError> {
    let response = client
        .get(Url::parse(API_BASE).and_then(|url| url.join("user/repos"))?)
        .header(header::AUTHORIZATION, format!("Bearer {access_token}"))
        .send()
        .await?;

    GithubApiError::match_status_code(response.status())?;

    response
        .json::<Vec<RepositoryResponse>>()
        .await
        .map_err(GithubApiError::Response)
}
