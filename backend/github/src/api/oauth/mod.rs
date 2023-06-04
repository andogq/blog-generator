mod access_token_response;
mod scope;

use reqwest::{Client, Url};
use serde_json::json;
use url::ParseError;

use super::GithubApiError;
pub use access_token_response::*;
pub use scope::*;

pub struct OauthApi {
    api_base: Url,
    client: Client,
    client_id: String,
    client_secret: String,
}

impl OauthApi {
    pub fn new(client: &Client, api_base: &Url, client_id: &str, client_secret: &str) -> Self {
        Self {
            api_base: api_base.clone(),
            client: client.clone(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        }
    }

    pub async fn get_access_token(
        &self,
        code: &str,
    ) -> Result<AuthAccessTokenResponse, GithubApiError> {
        let request = self
            .client
            .post(self.api_base.join("access_token")?)
            .json(&json!({
                "client_id": self.client_id,
                "client_secret": self.client_secret,
                "code": code
            }))
            .build()?;

        let response = self.client.execute(request).await?;

        GithubApiError::match_status_code(response.status())?;

        response
            .json::<AuthAccessTokenResponse>()
            .await
            .map_err(GithubApiError::Response)
    }

    pub fn generate_redirect_url(
        &self,
        scopes: &[Scope],
        redirect_url: &str,
    ) -> Result<Url, ParseError> {
        let mut url = self.api_base.join("authorize")?;
        url.query_pairs_mut().extend_pairs([
            (
                "scope",
                scopes
                    .iter()
                    .map(String::from)
                    .collect::<Vec<_>>()
                    .join(" ")
                    .as_str(),
            ),
            ("client_id", &self.client_id),
            ("redirect_uri", redirect_url),
        ]);

        Ok(url)
    }
}
