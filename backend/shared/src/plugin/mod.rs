use axum::Router;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;

mod auth;
mod projects;
mod user;

pub use auth::*;
pub use projects::*;
pub use user::*;

pub struct AuthTokenPayload {
    source: String,
    username: String,
    auth_token: String,
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

#[derive(Default)]
pub struct PluginCollection {
    pub auth: Vec<(String, Box<dyn AuthPlugin>)>,
    pub user: Vec<(String, Box<dyn UserPlugin>)>,
    pub project: Vec<(String, Box<dyn ProjectsPlugin>)>,
}

impl PluginCollection {
    pub fn build_router(
        &mut self,
        source_identifier: &str,
        save_auth_token: SaveAuthToken,
    ) -> Router {
        std::mem::take(&mut self.auth).into_iter().fold(
            Router::new(),
            |router, (identifier, auth_plugin)| {
                router.nest(
                    &format!("/{}", identifier),
                    auth_plugin.register_routes(source_identifier, save_auth_token.clone()),
                )
            },
        )
    }
}

impl FromIterator<PluginCollection> for PluginCollection {
    fn from_iter<T: IntoIterator<Item = PluginCollection>>(iter: T) -> Self {
        iter.into_iter()
            .reduce(|mut combined, mut source| {
                combined
                    .auth
                    .extend(std::mem::take(&mut source.auth).into_iter());
                combined
                    .user
                    .extend(std::mem::take(&mut source.user).into_iter());
                combined
                    .project
                    .extend(std::mem::take(&mut source.project).into_iter());

                combined
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("missing environment variable {0}")]
    MissingEnvVar(String),
    #[error("invalid url: {0}")]
    UrlParse(#[from] url::ParseError),
}
