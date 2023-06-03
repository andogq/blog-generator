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
use serde_json::json;
use shared::source::auth::{AuthIdentifier, AuthSource};
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    rest_api::responses::{GetUserResponse, OAuthAccessTokenResponse},
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
    #[error("non-200 status code from Github: {0}")]
    GithubError(StatusCode),
    #[error("unknown body response from Github: {0}")]
    InvalidBody(reqwest::Error),
    #[error("channel error")]
    Channel,
    #[error("url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
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
    let request = state
        .client
        .post(state.config.oauth_base.join("access_token")?)
        .json(&json!({
            "client_id": *state.config.client_id,
            "client_secret": *state.config.client_secret,
            "code": params.code
        }))
        .build()
        .map_err(OAuthHandlerError::Reqwest)?;

    let response = state
        .client
        .execute(request)
        .await
        .map_err(OAuthHandlerError::Reqwest)?;

    if !response.status().is_success() {
        return Err(OAuthHandlerError::GithubError(response.status()));
    }

    let access_token = response
        .json::<OAuthAccessTokenResponse>()
        .await
        .map_err(OAuthHandlerError::InvalidBody)?
        .access_token;

    // Get user information
    let response = state
        .client
        .get(state.config.api_base.join("user")?)
        .header(header::AUTHORIZATION, format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(OAuthHandlerError::Reqwest)?;

    if !response.status().is_success() {
        return Err(OAuthHandlerError::GithubError(response.status()));
    }

    let user_info = response
        .json::<GetUserResponse>()
        .await
        .map_err(OAuthHandlerError::InvalidBody)?;

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
    let mut redirect_url = state.config.oauth_base.join("authorize")?;
    redirect_url.query_pairs_mut().extend_pairs([
        ("scope", ["read:user", "repo"].join(" ").as_str()),
        ("client_id", &state.config.client_id),
        ("redirect_uri", "http://localhost:3000/auth/github/oauth"),
    ]);

    Ok(Redirect::temporary(redirect_url.as_ref()))
}
