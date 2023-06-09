use axum::Router;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use url::Url;

use super::PluginIdentifier;

pub trait AuthPlugin {
    fn register_routes(
        &self,
        source_identifier: &str,
        redirect_base: &Url,
        save_auth_token: SaveAuthToken,
    ) -> Router<()>;
    fn get_identifier(&self) -> PluginIdentifier;
}

#[derive(Clone)]
pub struct AuthTokenPayload {
    pub source: String,
    pub username: String,
    pub auth_token: String,
}
impl AuthTokenPayload {
    pub fn new(source: &str, username: &str, auth_token: &str) -> Self {
        Self {
            source: source.to_string(),
            username: username.to_string(),
            auth_token: auth_token.to_string(),
        }
    }

    pub fn to_key_value(self) -> ((String, String), String) {
        ((self.source, self.username), self.auth_token)
    }
}

pub type SaveAuthToken = UnboundedSender<AuthTokenPayload>;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("missing environment variable {0}")]
    MissingEnvVar(String),
    #[error("invalid url: {0}")]
    UrlParse(#[from] url::ParseError),
}
