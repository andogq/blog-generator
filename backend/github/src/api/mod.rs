use reqwest::StatusCode;
use shared::plugin::PluginError;
use thiserror::Error;

pub mod oauth;
pub mod rest;

#[derive(Debug, Error)]
pub enum GithubApiError {
    #[error("unable to parse url: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("problem with request: {0}")]
    Request(#[from] reqwest::Error),
    #[error("authentication required")]
    AuthenticationRequired,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("unknown status code: {0}")]
    StatusCode(StatusCode),
    #[error("unable to parse response: {0}")]
    Response(reqwest::Error),
}

impl GithubApiError {
    pub fn match_status_code(status_code: StatusCode) -> Result<(), GithubApiError> {
        match status_code {
            StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED => Err(GithubApiError::AuthenticationRequired),
            StatusCode::FORBIDDEN => Err(GithubApiError::Forbidden),
            StatusCode::NOT_FOUND => Err(GithubApiError::NotFound),
            status => Err(GithubApiError::StatusCode(status)),
        }
    }
}

impl From<GithubApiError> for PluginError {
    fn from(error: GithubApiError) -> Self {
        match error {
            GithubApiError::UrlParse(_) => Self::Internal,
            GithubApiError::Request(_) => Self::Internal,
            GithubApiError::AuthenticationRequired => Self::NotAuthorised,
            GithubApiError::Forbidden => Self::NotAuthorised,
            GithubApiError::NotFound => Self::NotFound,
            GithubApiError::StatusCode(status_code) => {
                if status_code.is_server_error() {
                    Self::External
                } else {
                    Self::Internal
                }
            }
            GithubApiError::Response(_) => Self::Internal,
        }
    }
}
