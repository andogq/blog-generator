mod repository_response;

use reqwest::{header, Client, Url};

use crate::api::GithubApiError;
pub use repository_response::RepositoryResponse;

pub struct RepositoriesApi {
    client: Client,
    api_base: Url,
}

impl RepositoriesApi {
    pub fn new(client: &Client, api_base: &Url) -> Self {
        Self {
            client: client.clone(),
            api_base: api_base.clone(),
        }
    }

    pub async fn list(
        &self,
        access_token: &str,
    ) -> Result<Vec<RepositoryResponse>, GithubApiError> {
        let response = self
            .client
            .get(self.api_base.join("user/repos")?)
            .header(header::AUTHORIZATION, format!("Bearer {access_token}"))
            .send()
            .await?;

        GithubApiError::match_status_code(response.status())?;

        response
            .json::<Vec<RepositoryResponse>>()
            .await
            .map_err(GithubApiError::Response)
    }
}
