use std::{collections::HashMap, sync::Arc};

use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Json;
use axum::{routing::get, Router, Server};
use reqwest::StatusCode;
use serde::Deserialize;
use shared::environment::Environment;
use shared::source::{Source, SourceCollection, SourceError};
use thiserror::Error;
use tokio::{
    sync::{mpsc::unbounded_channel, RwLock},
    task,
};

use github::Github;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug, Error)]
enum BackendError {
    #[error("loading env file with dotenvy failed: {0}")]
    DotEnv(#[from] dotenvy::Error),
    #[error("source error: {0}")]
    Source(#[from] SourceError),
}

#[derive(Deserialize)]
struct UserRequestPath {
    username: String,
}

#[tokio::main]
async fn main() -> Result<(), BackendError> {
    #[cfg(feature = "dev")]
    {
        // Load env variables from file
        println!("Starting in dev mode");

        dotenvy::from_filename("../.env.dev")?;
    }

    let environment = {
        let mut environment = std::env::vars().collect::<Environment>();
        environment.extend([("USER_AGENT".to_string(), APP_USER_AGENT.to_string())].into_iter());
        environment
    };

    let (save_auth_token, mut save_auth_token_rx) = unbounded_channel::<(String, String, String)>();

    let mut sources = [Box::new(Github::from_environment(&environment)?) as Box<dyn Source>]
        .into_iter()
        .map(|source| source.get_sources())
        .collect::<SourceCollection>();
    let auth_router = sources.build_router(save_auth_token);

    let authentication_storage = Arc::new(RwLock::new(HashMap::<(String, String), String>::new()));

    {
        let authentication_storage = Arc::clone(&authentication_storage);
        task::spawn(async move {
            while let Some((identifier, username, auth_token)) = save_auth_token_rx.recv().await {
                println!("New auth: {identifier}, {username}:{auth_token}");
                authentication_storage
                    .write()
                    .await
                    .insert((identifier, username), auth_token);
            }
        });
    }

    let router = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest(
            "/user",
            sources
                .user
                .into_iter()
                .fold(Router::new(), |router, (identifier, source)| {
                    let source = Arc::new(source);
                    let authentication_storage = Arc::clone(&authentication_storage);

                    router.route(
                        &format!("/{identifier}/:username"),
                        get(|Path(params): Path<UserRequestPath>| async move {
                            // Extract user authentication token
                            let auth_token = authentication_storage
                                .read()
                                .await
                                .get(&(identifier.to_string(), params.username.clone()))
                                .cloned();

                            if let Some(auth_token) = auth_token {
                                Json(source.get_user(&params.username, &auth_token).await)
                                    .into_response()
                            } else {
                                StatusCode::NOT_FOUND.into_response()
                            }
                        }),
                    )
                }),
        )
        .nest("/auth", auth_router);

    println!("Starting server on port 3000");
    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}
