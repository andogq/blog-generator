use axum::Router;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;

mod auth;
mod projects;
mod user;

pub use auth::*;
pub use projects::*;
pub use user::*;

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
        save_auth_token: UnboundedSender<(String, String, String)>,
    ) -> Router {
        std::mem::take(&mut self.auth).into_iter().fold(
            Router::new(),
            |router, (identifier, auth_source)| {
                router.nest(
                    &format!("/{}", identifier),
                    auth_source.register_routes(source_identifier, save_auth_token.clone()),
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
