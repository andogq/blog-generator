use axum::Router;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;

use self::auth::AuthIdentifier;
pub use self::{auth::AuthSource, project::ProjectsSource, user::UserSource};

pub mod auth;
pub mod project;
pub mod user;

#[derive(Default)]
pub struct SourceCollection {
    pub auth: Vec<Box<dyn AuthSource>>,
    pub user: Vec<Box<dyn UserSource>>,
    pub project: Vec<Box<dyn ProjectsSource>>,
}

impl SourceCollection {
    pub fn build_router(
        &mut self,
        user_agent: &str,
        save_auth_token: UnboundedSender<(AuthIdentifier, String, String)>,
    ) -> Router {
        std::mem::take(&mut self.auth)
            .into_iter()
            .fold(Router::new(), |router, auth_source| {
                router.nest(
                    &format!("/{}", *auth_source.get_identifier()),
                    auth_source.register_routes(user_agent, save_auth_token.clone()),
                )
            })
    }
}

impl FromIterator<SourceCollection> for SourceCollection {
    fn from_iter<T: IntoIterator<Item = SourceCollection>>(iter: T) -> Self {
        iter.into_iter()
            .reduce(|mut combined, mut source| {
                combined.auth.append(&mut source.auth);

                combined
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("missing environment variable {0}")]
    MissingEnvVar(String),
    #[error("invalid url: {0}")]
    UrlParse(#[from] url::ParseError),
}

pub trait Source {
    fn get_sources(&self) -> SourceCollection;
}
