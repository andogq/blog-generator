use reqwest::{Client, Url};
use serde::Deserialize;
use serde_json::json;

use crate::api::GithubApiError;

#[derive(Deserialize)]
pub struct OAuthAccessTokenResponse {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
}

pub async fn get_access_token(
    api_base: &Url,
    client: Client,
    client_id: &str,
    client_secret: &str,
    code: &str,
) -> Result<OAuthAccessTokenResponse, GithubApiError> {
    let request = client
        .post(api_base.join("access_token")?)
        .json(&json!({
            "client_id": client_id,
            "client_secret": client_secret,
            "code": code
        }))
        .build()?;

    let response = client.execute(request).await?;

    GithubApiError::match_status_code(response.status())?;

    response
        .json::<OAuthAccessTokenResponse>()
        .await
        .map_err(GithubApiError::Response)
}
