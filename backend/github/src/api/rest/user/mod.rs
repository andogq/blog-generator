mod user_response;

use reqwest::{header, Client, Url};

use crate::api::GithubApiError;
pub use user_response::*;

pub struct UserApi {
    client: Client,
    api_base: Url,
}

impl UserApi {
    pub fn new(client: &Client, api_base: &Url) -> Self {
        Self {
            client: client.clone(),
            api_base: api_base.clone(),
        }
    }

    pub async fn get(&self, access_token: &str) -> Result<UserResponse, GithubApiError> {
        let response = self
            .client
            .get(self.api_base.join("user")?)
            .header(header::AUTHORIZATION, format!("Bearer {access_token}"))
            .send()
            .await?;

        GithubApiError::match_status_code(response.status())?;

        response
            .json::<UserResponse>()
            .await
            .map_err(GithubApiError::Response)
    }
}
