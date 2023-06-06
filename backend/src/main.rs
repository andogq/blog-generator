use std::{collections::HashMap, sync::Arc};

use axum::extract::Path;
use axum::response::IntoResponse;
use axum::{routing::get, Router, Server};
use reqwest::StatusCode;
use sea_orm::{ActiveModelTrait, ConnectOptions, Database, DbErr, EntityTrait, Set};
use serde::Deserialize;
use shared::environment::Environment;
use shared::plugin::{AuthTokenPayload, SourceError};
use shared::source::Source;
use thiserror::Error;
use tokio::{sync::mpsc::unbounded_channel, task};

use entities::{user, user_source, UserSource};
use github::Github;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug, Error)]
enum BackendError {
    #[error("loading env file with dotenvy failed: {0}")]
    DotEnv(#[from] dotenvy::Error),
    #[error("source error: {0}")]
    Source(#[from] SourceError),
    #[error("DB error: {0}")]
    Db(#[from] DbErr),
}

#[derive(Deserialize)]
struct PluginPathParams {
    request_type: String,
    source_identifier: String,
    plugin_identifier: String,
    username: String,
}

#[tokio::main]
async fn main() -> Result<(), BackendError> {
    #[cfg(feature = "dev")]
    {
        // Load env variables from file
        println!("Starting in dev mode");

        dotenvy::from_filename("../.env")?;
    }

    let environment = {
        let mut environment = std::env::vars().collect::<Environment>();
        environment.extend([("USER_AGENT".to_string(), APP_USER_AGENT.to_string())].into_iter());
        environment
    };

    let mut opt = ConnectOptions::new(environment.get("DATABASE_URL").unwrap().clone());
    opt.max_connections(100)
        .min_connections(5)
        .sqlx_logging(true);
    let db = Arc::new(Database::connect(opt).await?);

    let (save_auth_token, mut save_auth_token_rx) = unbounded_channel::<AuthTokenPayload>();

    let (auth_plugins, plugins) =
        [(
            "github".to_string(),
            Github::from_environment(&environment)?,
        )]
        .into_iter()
        .fold(
            (Vec::new(), HashMap::new()),
            |(mut auth_plugins, mut plugins), (source_identifier, source)| {
                auth_plugins.extend(source.get_auth_plugins().into_iter().map(
                    |(plugin_identifier, plugin)| {
                        (source_identifier.clone(), plugin_identifier, plugin)
                    },
                ));
                plugins.extend(source.get_plugins().into_iter().map(
                    |(plugin_identifier, plugin)| {
                        (
                            (
                                plugin.request_type(),
                                source_identifier.clone(),
                                plugin_identifier,
                            ),
                            plugin,
                        )
                    },
                ));

                (auth_plugins, plugins)
            },
        );

    {
        let db = db.clone();
        task::spawn(async move {
            while let Some(auth_token) = save_auth_token_rx.recv().await {
                println!("Adding {}: {}", auth_token.source, auth_token.username);

                // Create user in DB
                let user = user::ActiveModel {
                    ..Default::default()
                };

                if let Ok(user) = user.insert(db.as_ref()).await {
                    // Add to user source
                    let user_source = user_source::ActiveModel {
                        user_id: Set(user.id),
                        site: Set(auth_token.source),
                        username: Set(auth_token.username),
                        token: Set(auth_token.auth_token),
                        ..Default::default()
                    };

                    if let Ok(user_source) = user_source.insert(db.as_ref()).await {
                        println!("Successfully created");
                        dbg!(user_source);
                    } else {
                        eprintln!("Problem creating user source");
                    }
                } else {
                    eprintln!("Problem creating user");
                }
            }
        });
    }

    let plugins = Arc::new(plugins);

    let router = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route(
            "/:request_type/:source_identifier/:plugin_identifier/:username",
            get({
                let db = db.clone();
                let plugins = plugins.clone();

                |Path(params): Path<PluginPathParams>| async move {
                    // Attempt to find authentication for the user and source
                    let user_source =
                        UserSource::find_by_id((params.username, params.source_identifier.clone()))
                            .one(db.as_ref())
                            .await
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                            .and_then(|user_source| user_source.ok_or(StatusCode::UNAUTHORIZED));

                    // Attempt to find plugin to run
                    let plugin = plugins
                        .get(&(
                            params.request_type.to_string(),
                            params.source_identifier.to_string(),
                            params.plugin_identifier.to_string(),
                        ))
                        .ok_or(StatusCode::NOT_FOUND);

                    // If user and plugin found, run it
                    match (user_source, plugin) {
                        (Ok(user_source), Ok(plugin)) => plugin
                            .get_data(&user_source.username, &user_source.token)
                            .await
                            .into_response(),
                        (Err(e), _) | (_, Err(e)) => e.into_response(),
                    }
                }
            }),
        )
        .nest(
            "/auth",
            auth_plugins.into_iter().fold(
                Router::new(),
                |router, (source_identifier, plugin_identifier, plugin)| {
                    router.nest(
                        &format!("/{source_identifier}/{plugin_identifier}"),
                        plugin.register_routes(&source_identifier, save_auth_token.clone()),
                    )
                },
            ),
        );

    println!("Starting server on port 3000");
    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}
