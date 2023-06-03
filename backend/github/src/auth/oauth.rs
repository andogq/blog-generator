use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, HeaderValue},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use reqwest::{
    header::{self, InvalidHeaderValue},
    Client, StatusCode,
};
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;

use shared::source::auth::{AuthIdentifier, AuthSource};

use crate::{
    api::{
        oauth::{generate_redirect_url, get_access_token, Scope},
        rest::get_user,
        GithubApiError,
    },
    GithubConfig,
};

pub struct GithubOAuth {
    config: GithubConfig,
}

impl GithubOAuth {
    pub fn new(config: &GithubConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

#[derive(Clone)]
struct AuthState {
    client: Client,
    save_auth_token: UnboundedSender<(AuthIdentifier, String, String)>,
    config: Arc<GithubConfig>,
    identifier: Arc<AuthIdentifier>,
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

impl AuthSource for GithubOAuth {
    fn get_identifier(&self) -> AuthIdentifier {
        AuthIdentifier::new("github")
    }

    fn register_routes(
        &self,
        user_agent: &str,
        save_auth_token: UnboundedSender<(AuthIdentifier, String, String)>,
    ) -> Router {
        let state = AuthState {
            client: Client::builder()
                .default_headers(
                    [(header::ACCEPT, "application/vnd.github+json")]
                        .into_iter()
                        .map(|(header, value)| Ok((header, HeaderValue::from_str(value)?)))
                        .collect::<Result<HeaderMap, InvalidHeaderValue>>()
                        .unwrap(),
                )
                .user_agent(user_agent)
                .build()
                .unwrap(),
            save_auth_token,
            config: Arc::new(self.config.clone()),
            identifier: Arc::new(self.get_identifier()),
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
        .send((
            state.identifier.as_ref().clone(),
            user_info.login,
            access_token,
        ))
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
