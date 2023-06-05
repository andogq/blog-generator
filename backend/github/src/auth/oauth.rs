use std::sync::Arc;

use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use reqwest::StatusCode;
use serde::Deserialize;
use shared::plugin::{AuthPlugin, AuthTokenPayload, SaveAuthToken};
use thiserror::Error;

use crate::api::{
    oauth::{OauthApi, Scope},
    rest::RestApi,
    GithubApiError,
};

pub struct GithubOAuth {
    rest_api: Arc<RestApi>,
    oauth_api: Arc<OauthApi>,
}

impl GithubOAuth {
    pub fn new(rest_api: &Arc<RestApi>, oauth_api: &Arc<OauthApi>) -> Self {
        Self {
            rest_api: Arc::clone(rest_api),
            oauth_api: Arc::clone(oauth_api),
        }
    }
}

#[derive(Clone)]
struct AuthState {
    save_auth_token: SaveAuthToken,
    source_identifier: Arc<String>,
    rest_api: Arc<RestApi>,
    oauth_api: Arc<OauthApi>,
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

impl AuthPlugin for GithubOAuth {
    fn register_routes(
        &self,
        source_identifier: &str,
        save_auth_token: SaveAuthToken,
    ) -> Router<()> {
        let state = AuthState {
            save_auth_token,
            source_identifier: Arc::new(source_identifier.to_string()),
            rest_api: Arc::clone(&self.rest_api),
            oauth_api: Arc::clone(&self.oauth_api),
        };

        Router::new()
            .route(
                "/oauth",
                get({
                    let state = state.clone();
                    move |params| handle_oauth(state, params)
                }),
            )
            .route("/redirect", get(move || handle_redirect(state)))
    }
}

#[derive(Deserialize)]
struct OauthQueryParams {
    code: String,
}

async fn handle_oauth(
    state: AuthState,
    params: Query<OauthQueryParams>,
) -> Result<StatusCode, OAuthHandlerError> {
    let access_token = state
        .oauth_api
        .get_access_token(&params.code)
        .await?
        .access_token;

    let user_info = state.rest_api.user.get(&access_token).await?;

    state
        .save_auth_token
        .send(AuthTokenPayload::new(
            &state.source_identifier,
            &user_info.login,
            &access_token,
        ))
        .map_err(|_| OAuthHandlerError::Channel)?;

    Ok(StatusCode::OK)
}

async fn handle_redirect(state: AuthState) -> Result<Redirect, OAuthHandlerError> {
    state
        .oauth_api
        .generate_redirect_url(
            &[Scope::ReadUser, Scope::Repo],
            "http://localhost:3000/auth/github/oauth/oauth",
        )
        .map(|url| Redirect::temporary(url.as_ref()))
        .map_err(OAuthHandlerError::UrlParse)
}
