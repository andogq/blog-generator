use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json, Router, Server,
};
use providers::github::GithubProviderError;
use reqwest::StatusCode;
use thiserror::Error;

use crate::providers::{github, Provider};

mod providers;

#[derive(Debug, Error)]
enum BackendError {
    #[error("loading env file with dotenvy failed: {0}")]
    DotEnv(#[from] dotenvy::Error),
    #[error("config generation failed: {0}")]
    Config(#[from] ConfigError),
    #[error("Github error: {0}")]
    Github(#[from] GithubProviderError),
}

#[derive(Debug, Error)]
enum ConfigError {
    #[error("missing env var {0}")]
    MissingEnvVar(String),
}
struct Config {
    github_token: String,
    github_client_id: String,
    github_api: String,
}
impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        macro_rules! var {
            ($name:expr) => {
                std::env::var($name).map_err(|_| ConfigError::MissingEnvVar($name.to_string()))?
            };
        }

        Ok(Self {
            github_token: var!("GITHUB_CLIENT_SECRET"),
            github_client_id: var!("GITHUB_CLIENT_ID"),
            github_api: var!("GITHUB_API"),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), BackendError> {
    #[cfg(feature = "dev")]
    {
        // Load env variables from file
        println!("Starting in dev mode");

        dotenvy::from_filename("../.env.dev")?;
    }

    let config = Config::new()?;
    println!("Config successfully loaded");

    let providers: HashMap<String, Box<dyn Provider>> = [(
        "github".to_string(),
        Box::new(github::GithubProvider::new(
            &config.github_api,
            &config.github_token,
        )?) as Box<dyn Provider>,
    )]
    .into_iter()
    .collect();

    let providers = Arc::new(providers);

    let router = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route(
            "/:provider/:user",
            get(
                |Path((provider_name, user)): Path<(String, String)>,
                 providers: State<Arc<HashMap<String, Box<dyn Provider>>>>| async move {
                    if let Some(provider) = providers.get(provider_name.as_str()) {
                        match provider.get_user(&user).await {
                            Ok(Some(user_info)) => Json(user_info).into_response(),
                            Ok(None) => StatusCode::NOT_FOUND.into_response(),
                            Err(e) => {
                                eprintln!("{e}");
                                StatusCode::INTERNAL_SERVER_ERROR.into_response()
                            }
                        }
                    } else {
                        StatusCode::NOT_FOUND.into_response()
                    }
                },
            ),
        )
        .with_state(providers);

    println!("Starting server on port 3000");
    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}
