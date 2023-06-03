use serde::Deserialize;

#[derive(Deserialize)]
pub struct OAuthAccessTokenResponse {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
}
