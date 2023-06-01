use crate::providers::{ExternalSite, Project, RepoStats, UserInformation};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetUserResponse {
    pub login: String,
    avatar_url: String,
    html_url: String,
    name: Option<String>,
    company: Option<String>,
    blog: Option<String>,
    location: Option<String>,
    email: Option<String>,
    bio: Option<String>,
    twitter_username: Option<String>,
}
impl From<GetUserResponse> for UserInformation {
    fn from(response: GetUserResponse) -> Self {
        Self {
            name: response.name,
            avatar: response.avatar_url,
            bio: response.bio,
            location: response.location,
            email: response.email,
            links: [
                (ExternalSite::Github, Some(response.html_url)),
                (
                    ExternalSite::Twitter,
                    response
                        .twitter_username
                        .map(|username| format!("https://twitter.com/{username}")),
                ),
            ]
            .into_iter()
            .filter_map(|(site, url)| url.map(|url| (site, url)))
            .collect(),
            blog: response.blog,
            company: response.company,
        }
    }
}

#[derive(Deserialize)]
pub struct OAuthAccessTokenResponse {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
}

#[derive(Deserialize)]
pub struct RepositoryResponse {
    name: String,
    private: bool,
    html_url: String,
    description: Option<String>,
    homepage: Option<String>,
    stargazers_count: usize,
    watchers_count: usize,
    forks_count: usize,
    open_issues_count: usize,
    topics: Vec<String>,
    language: String,
}

impl From<RepositoryResponse> for Project {
    fn from(repo: RepositoryResponse) -> Self {
        Self {
            name: repo.name,
            description: repo.description,
            url: repo.homepage,
            repo_url: (!repo.private).then_some(repo.html_url),
            repo_stats: Some(RepoStats {
                stars: repo.stargazers_count,
                forks: repo.forks_count,
                watchers: repo.watchers_count,
                issues: repo.open_issues_count,
            }),
            tags: repo.topics,
            languages: Some(vec![repo.language]),
        }
    }
}

#[derive(Deserialize)]
pub struct SearchRepositoriesResponse {
    pub total_count: usize,
    pub incomplete_results: bool,
    pub items: Vec<RepositoryResponse>,
}
