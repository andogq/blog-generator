use std::{collections::HashMap, sync::Arc};

use axum::extract::Path;
use axum::{routing::get, Router, Server};
use serde::Deserialize;
use shared::environment::Environment;
use shared::source::auth::AuthIdentifier;
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
    source: String,
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

    let environment: Environment = std::env::vars().collect();

    let mut sources = [Box::new(Github::from_environment(&environment)?) as Box<dyn Source>]
        .into_iter()
        .map(|source| source.get_sources())
        .collect::<SourceCollection>();

    let authentication_storage = Arc::new(RwLock::new(
        HashMap::<(AuthIdentifier, String), String>::new(),
    ));

    let (save_auth_token, mut save_auth_token_rx) =
        unbounded_channel::<(AuthIdentifier, String, String)>();
    {
        let authentication_storage = Arc::clone(&authentication_storage);
        task::spawn(async move {
            while let Some((identifier, username, auth_token)) = save_auth_token_rx.recv().await {
                println!("New auth: {username}:{auth_token}");
                authentication_storage
                    .write()
                    .await
                    .insert((identifier, username), auth_token);
            }
        });
    }

    let router = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route(
            "/:source/:username",
            get(|Path(params): Path<UserRequestPath>| async move {
                // todo
                let auth_token = authentication_storage
                    .read()
                    .await
                    .get(&(AuthIdentifier::new(&params.source), params.username.clone()))
                    .cloned();

                format!(
                    "Responding with info from {} for {}: {auth_token:?}",
                    params.source, params.username
                )
            }),
        )
        .nest(
            "/auth",
            sources.build_router(APP_USER_AGENT, save_auth_token),
        );

    println!("Starting server on port 3000");
    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}
