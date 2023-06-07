use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use std::{fmt::Display, ops::Deref};
use thiserror::Error;

mod auth;
mod data;
mod response;

pub use auth::*;
pub use data::*;
pub use response::*;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct PluginIdentifier(String);
impl PluginIdentifier {
    pub fn new(identifier: &str) -> Self {
        Self(identifier.to_string())
    }
}
impl Deref for PluginIdentifier {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Display for PluginIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

type UserPlugin = Box<dyn DataPlugin<D = UserResponse>>;
type ProjectsPlugin = Box<dyn DataPlugin<D = ProjectsResponse>>;

#[derive(Serialize)]
#[serde(untagged)]
pub enum PluginResponse {
    User(UserResponse),
    Projects(ProjectsResponse),
}
impl IntoResponse for PluginResponse {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

pub enum Plugin {
    User(UserPlugin),
    Projects(ProjectsPlugin),
}

impl Plugin {
    pub fn request_type(&self) -> String {
        match self {
            Self::User(_) => "user",
            Self::Projects(_) => "projects",
        }
        .to_string()
    }
    pub async fn get_data(
        &self,
        username: &str,
        auth_token: &str,
    ) -> Result<PluginResponse, PluginError> {
        macro_rules! expand_plugins {
            ($($plugin:ident),*) => {
                match self {
                    $(
                      Self::$plugin(plugin) => plugin.get_data(username, auth_token)
                          .await
                          .map(PluginResponse::$plugin)
                    ),*
                }
            };
        }

        expand_plugins!(User, Projects)
    }

    pub fn get_identifier(&self) -> PluginIdentifier {
        match self {
            Self::User(plugin) => plugin.get_identifier(),
            Self::Projects(plugin) => plugin.get_identifier(),
        }
    }
}

pub trait ToPlugin {
    fn to_plugin(self) -> Plugin;
}

// Trait magic: https://stackoverflow.com/a/40408431
trait InnerToPlugin<P> {
    fn to_plugin(plugin: P) -> Plugin;
}

impl<P> ToPlugin for P
where
    P: DataPlugin,
    <P as DataPlugin>::D: InnerToPlugin<P>,
{
    fn to_plugin(self) -> Plugin {
        <<P as DataPlugin>::D as InnerToPlugin<P>>::to_plugin(self)
    }
}

macro_rules! impl_to_plugin {
    ($($plugin:ident: $response:ident),*) => {
        $(
            impl<P> InnerToPlugin<P> for $response
            where
                P: DataPlugin<D = $response> + 'static,
            {
                fn to_plugin(plugin: P) -> Plugin {
                    Plugin::$plugin(Box::new(plugin))
                }
            }
        )*
    };
}

impl_to_plugin!(User: UserResponse, Projects: ProjectsResponse);

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("requested resource is not found")]
    NotFound,
    #[error("not authorised to access requested resource")]
    NotAuthorised,
    #[error("an external provider could not fulfill the request")]
    External,
    #[error("an internal error occurred")]
    Internal,
}

impl IntoResponse for PluginError {
    fn into_response(self) -> axum::response::Response {
        match self {
            PluginError::NotFound => StatusCode::NOT_FOUND.into_response(),
            PluginError::NotAuthorised => StatusCode::UNAUTHORIZED.into_response(),
            PluginError::External => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            PluginError::Internal => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
