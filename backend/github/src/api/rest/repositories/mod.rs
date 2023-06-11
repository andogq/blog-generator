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

    pub async fn get_readme(
        &self,
        access_token: &str,
        user: &str,
        repo: &str,
        rendered: bool,
    ) -> Result<String, GithubApiError> {
        let response = self
            .client
            .get(self.api_base.join(&format!("repos/{user}/{repo}/readme"))?)
            .header(header::AUTHORIZATION, format!("Bearer {access_token}"))
            .header(
                header::ACCEPT,
                format!(
                    "application/vnd.github.{}",
                    if rendered { "html" } else { "raw" }
                ),
            )
            .send()
            .await?;

        GithubApiError::match_status_code(response.status())?;

        response.text().await.map_err(GithubApiError::Response)
    }
}
