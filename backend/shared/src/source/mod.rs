use std::collections::HashMap;

use axum::Router;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;

pub use self::{auth::AuthSource, project::ProjectsSource, user::UserSource};

pub mod auth;
pub mod project;
pub mod user;

#[derive(Default)]
pub struct SourceCollection {
    pub auth: Vec<(String, Box<dyn AuthSource>)>,
    pub user: Vec<(String, Box<dyn UserSource>)>,
    pub project: Vec<(String, Box<dyn ProjectsSource>)>,
}

impl SourceCollection {
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

impl FromIterator<SourceCollection> for SourceCollection {
    fn from_iter<T: IntoIterator<Item = SourceCollection>>(iter: T) -> Self {
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
pub enum SourceError {
    #[error("missing environment variable {0}")]
    MissingEnvVar(String),
    #[error("invalid url: {0}")]
    UrlParse(#[from] url::ParseError),
}

pub trait Source {
    fn get_sources(&self) -> SourceCollection;
}
