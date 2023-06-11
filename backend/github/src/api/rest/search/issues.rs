use reqwest::{header, Client, Url};
use serde::Deserialize;
use shared::plugin::PostResponse;

use crate::api::GithubApiError;

use super::SearchResponse;

pub struct SearchIssuesApi {
    client: Client,
    api_base: Url,
}

#[derive(Deserialize)]
pub struct PartialUserResponse {
    pub login: String,
}

#[derive(Deserialize)]
pub struct LabelResponse {
    pub name: String,
    pub color: String,
}

#[derive(Deserialize)]
pub struct IssueResponse {
    pub html_url: String,
    pub number: usize,
    pub title: String,
    pub user: PartialUserResponse,
    pub labels: Vec<LabelResponse>,
    pub state: String,
    pub assignee: Option<PartialUserResponse>,
    pub comments: usize,
    pub created_at: String,
    pub updated_at: String,
    pub body: String,
}

impl From<IssueResponse> for PostResponse {
    fn from(issue: IssueResponse) -> Self {
        PostResponse {
            number: issue.number,
            title: issue.title,
            body: issue.body,
            created_at: issue.created_at,
            updated_at: issue.updated_at,
            original_link: issue.html_url,
        }
    }
}

pub struct SearchIssuesBuilder {
    client: Client,
    api_base: Url,
    access_token: String,

    repo: Option<String>,
    labels: Option<Vec<String>>,
    open: Option<bool>,
}

impl SearchIssuesBuilder {
    pub fn repo(mut self, repo: &str) -> Self {
        self.repo = Some(repo.to_string());

        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.labels = Some({
            let mut labels = self.labels.unwrap_or_default();
            labels.push(label.to_string());
            labels
        });

        self
    }

    pub fn open(mut self) -> Self {
        self.open = Some(true);
        self
    }

    pub fn closed(mut self) -> Self {
        self.open = Some(false);
        self
    }

    pub async fn search(self) -> Result<Vec<IssueResponse>, GithubApiError> {
        let query = [
            Some("is:issue".to_string()),
            self.repo.map(|repo| format!("repo:{repo}")),
            self.labels.map(|labels| {
                labels
                    .into_iter()
                    .map(|label| format!("label:{label}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            }),
            self.open
                .map(|open| format!("is:{}", if open { "open" } else { "closed" })),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");

        let response = self
            .client
            .get({
                let mut url = self.api_base.join("search/issues")?;
                url.query_pairs_mut().append_pair("q", &query);
                url
            })
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", self.access_token),
            )
            .send()
            .await?;

        GithubApiError::match_status_code(response.status())?;

        response
            .json::<SearchResponse<IssueResponse>>()
            .await
            .map(|results| results.items)
            .map_err(GithubApiError::Response)
    }
}

impl SearchIssuesApi {
    pub fn new(client: &Client, api_base: &Url) -> Self {
        Self {
            client: client.clone(),
            api_base: api_base.clone(),
        }
    }

    pub fn builder(&self, access_token: &str) -> SearchIssuesBuilder {
        SearchIssuesBuilder {
            client: self.client.clone(),
            api_base: self.api_base.clone(),
            access_token: access_token.to_string(),
            repo: None,
            labels: None,
            open: None,
        }
    }
}
