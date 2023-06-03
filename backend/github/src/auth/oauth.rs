use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;

use shared::source::{auth::AuthSource, IdentifiableSource};

use crate::{
    api::{
        oauth::{generate_redirect_url, get_access_token, Scope},
        rest::get_user,
        GithubApiError,
    },
    GithubConfig,
};

pub struct GithubOAuth {
    identifier: String,
    config: GithubConfig,
    client: Client,
}

impl GithubOAuth {
    pub fn new(identifier: &str, config: &GithubConfig, client: &Client) -> Self {
        Self {
            identifier: identifier.to_string(),
            config: config.clone(),
            client: client.clone(),
        }
    }
}

#[derive(Clone)]
struct AuthState {
    client: Client,
    save_auth_token: UnboundedSender<(String, String, String)>,
    config: Arc<GithubConfig>,
    identifier: Arc<String>,
}

#[derive(Debug, Error)]
enum OAuthHandlerError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("channel error")]
    Channel,
    #[error("url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Github API error: {0}")]
    GithubApi(#[from] GithubApiError),
}
impl IntoResponse for OAuthHandlerError {
    fn into_response(self) -> axum::response::Response {
        eprintln!("{self}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl IdentifiableSource for GithubOAuth {
    fn get_identifier(&self) -> String {
        self.identifier.to_string()
    }
}

impl AuthSource for GithubOAuth {
    fn register_routes(
        &self,
        save_auth_token: UnboundedSender<(String, String, String)>,
    ) -> Router {
        let state = AuthState {
            client: self.client.clone(),
            save_auth_token,
            config: Arc::new(self.config.clone()),
            identifier: Arc::new(self.identifier.clone()),
        };

        Router::new()
            .route("/oauth", get(handle_oauth))
            .route("/redirect", get(handle_redirect))
            .with_state(state)
    }
}

#[derive(Deserialize)]
struct OauthQueryParams {
    code: String,
}

async fn handle_oauth(
    State(state): State<AuthState>,
    params: Query<OauthQueryParams>,
) -> Result<StatusCode, OAuthHandlerError> {
    let access_token = get_access_token(
        state.client.clone(),
        &state.config.client_id,
        &state.config.client_secret,
        &params.code,
    )
    .await?
    .access_token;

    let user_info = get_user(state.client.clone(), &access_token).await?;

    state
        .save_auth_token
        .send((state.identifier.to_string(), user_info.login, access_token))
        .map_err(|_| OAuthHandlerError::Channel)?;

    Ok(StatusCode::OK)
}

async fn handle_redirect(State(state): State<AuthState>) -> Result<Redirect, OAuthHandlerError> {
    generate_redirect_url(
        &state.config.client_id,
        &[Scope::ReadUser, Scope::Repo],
        "http://localhost:3000/auth/github/oauth",
    )
    .map(|url| Redirect::temporary(url.as_ref()))
    .map_err(OAuthHandlerError::UrlParse)
}
