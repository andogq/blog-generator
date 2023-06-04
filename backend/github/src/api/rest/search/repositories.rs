use reqwest::{header, Client, Url};

use crate::api::{rest::RepositoryResponse, GithubApiError};

use super::SearchResponse;

pub struct SearchRepositoriesApi {
    client: Client,
    api_base: Url,
}

impl SearchRepositoriesApi {
    pub fn new(client: &Client, api_base: &Url) -> Self {
        Self {
            client: client.clone(),
            api_base: api_base.clone(),
        }
    }

    pub async fn by_topics(
        &self,
        access_token: &str,
        topics: &[String],
    ) -> Result<Vec<RepositoryResponse>, GithubApiError> {
        let topics = topics
            .iter()
            .map(|topic| format!("topic:{topic}"))
            .collect::<Vec<_>>()
            .join(" ");
        let query = format!("user:@me {topics}");

        let response = self
            .client
            .get({
                let mut url = self.api_base.join("search/repositories")?;

                url.query_pairs_mut().append_pair("q", &query);
                url
            })
            .header(header::AUTHORIZATION, format!("Bearer {access_token}"))
            .send()
            .await?;

        GithubApiError::match_status_code(response.status())?;

        response
            .json::<SearchResponse<RepositoryResponse>>()
            .await
            .map(|results| results.items)
            .map_err(GithubApiError::Response)
    }
}
