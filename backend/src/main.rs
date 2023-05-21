use axum::{routing::get, Router, Server};
use serde::Serialize;
use thiserror::Error;

#[derive(Serialize)]
struct UserInformation {
    name: String,
    github_url: String,
}

#[derive(Debug, Error)]
enum BackendError {
    #[error("loading env file with dotenvy failed: {0}")]
    DotEnv(#[from] dotenvy::Error),
    #[error("config generation failed: {0}")]
    Config(#[from] ConfigError),
}

#[derive(Debug, Error)]
enum ConfigError {
    #[error("missing env var {0}")]
    MissingEnvVar(String),
}
struct Config {
    github_token: String,
    github_client_id: String,
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

    let router = Router::new().route("/", get(|| async { "Hello, World!" }));

    println!("Starting server on port 3000");
    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}
