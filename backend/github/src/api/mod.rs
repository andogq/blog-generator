use reqwest::StatusCode;
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
            status => Err(GithubApiError::StatusCode(status)),
        }
    }
}
