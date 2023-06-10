use std::{collections::HashMap, sync::Arc};

use axum::extract::Path;
use axum::http::HeaderValue;
use axum::response::IntoResponse;
use axum::{routing::get, Router, Server};
use reqwest::{header, Method, StatusCode, Url};
use sea_orm::{ActiveModelTrait, ConnectOptions, Database, DbErr, EntityTrait, Set};
use serde::Deserialize;
use shared::environment::Environment;
use shared::plugin::{AuthTokenPayload, PluginIdentifier, SourceError};
use shared::source::{Source, SourceIdentifier};
use thiserror::Error;
use tokio::{sync::mpsc::unbounded_channel, task};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::{error, info, info_span, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use url::ParseError;

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
    #[error("missing environment variable: {0}")]
    Environment(String),
    #[error("unable to parse URL: {0}")]
    UrlParseError(#[from] ParseError),
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
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "api_aggregator=trace,tower_http=debug,axum::rejectuion=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting API aggregator");

    #[cfg(feature = "dev")]
    {
        // Load env variables from file
        info!("Starting in dev mode");

        dotenvy::from_filename("../.env.dev")?;
    }

    let environment = {
        let mut environment = std::env::vars().collect::<Environment>();
        environment.extend([("USER_AGENT".to_string(), APP_USER_AGENT.to_string())].into_iter());
        environment
    };
    info!("Loaded environment");

    let api_root: Url = environment
        .get("API_ROOT")
        .ok_or(BackendError::Environment("API_ROOT".to_string()))?
        .parse()?;
    info!("Running at {api_root}");

    let mut opt = ConnectOptions::new(environment.get("DATABASE_URL").unwrap().clone());
    opt.max_connections(100)
        .min_connections(5)
        .sqlx_logging(true);
    let db = Arc::new(Database::connect(opt).await?);
    info!("Connected to database");

    let (save_auth_token, mut save_auth_token_rx) = unbounded_channel::<AuthTokenPayload>();

    let (auth_plugins, plugins) = [Github::from_environment(&environment)?].into_iter().fold(
        (Vec::new(), HashMap::new()),
        |(mut auth_plugins, mut plugins), source| {
            auth_plugins.extend(
                source
                    .get_auth_plugins()
                    .into_iter()
                    .map(|plugin| (source.get_identifier(), plugin.get_identifier(), plugin)),
            );
            plugins.extend(source.get_plugins().into_iter().map(|plugin| {
                (
                    (
                        plugin.request_type(),
                        source.get_identifier(),
                        plugin.get_identifier(),
                    ),
                    plugin,
                )
            }));

            (auth_plugins, plugins)
        },
    );
    info!("Loaded plugins");

    {
        let db = db.clone();
        task::spawn(async move {
            while let Some(auth_token) = save_auth_token_rx.recv().await {
                let span = info_span!("save token", source = auth_token.source);
                let _guard = span.enter();

                // Create user in DB
                let user = user::ActiveModel {
                    ..Default::default()
                };

                match user.insert(db.as_ref()).await {
                    Ok(user) => {
                        info!("user created in DB");

                        // Add to user source
                        let user_source = user_source::ActiveModel {
                            user_id: Set(user.id),
                            site: Set(auth_token.source),
                            username: Set(auth_token.username),
                            token: Set(auth_token.auth_token),
                            ..Default::default()
                        };

                        match user_source.insert(db.as_ref()).await {
                            Ok(_) => {
                                info!("token saved in Db");
                            }
                            Err(e) => {
                                error!(message = "unable to save auth token to DB", error = ?e);
                            }
                        }
                    }
                    Err(e) => {
                        error!(message = "unable to create user in DB", error = ?e);
                    }
                }
            }
        });
    }

    let plugins = Arc::new(plugins);

    let router = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route(
            "/api/:request_type/:source_identifier/:plugin_identifier/:username",
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
                            SourceIdentifier::new(&params.source_identifier),
                            PluginIdentifier::new(&params.plugin_identifier),
                        ))
                        .ok_or(StatusCode::NOT_FOUND);

                    // If user and plugin found, run it
                    match (user_source, plugin) {
                        (Ok(user_source), Ok(plugin)) => {
                            let mut response = plugin
                                .get_data(&user_source.username, &user_source.token)
                                .await
                                .into_response();

                            response.headers_mut().extend([(
                                header::CACHE_CONTROL,
                                HeaderValue::from_str("public, max-age=86400").unwrap(),
                            )]);

                            response
                        }
                        (Err(e), _) | (_, Err(e)) => e.into_response(),
                    }
                }
            }),
        )
        .nest("/auth", {
            let auth_base = api_root.join("auth/")?;

            auth_plugins.into_iter().try_fold(
                Router::new(),
                |router,
                 (source_identifier, plugin_identifier, plugin)|
                 -> Result<Router, ParseError> {
                    let base = format!("{}/{}/", source_identifier, plugin_identifier);
                    let redirect_base = auth_base.join(&base)?;

                    Ok(router.nest(
                        &format!("/{base}"),
                        plugin.register_routes(
                            &source_identifier,
                            &redirect_base,
                            save_auth_token.clone(),
                        ),
                    ))
                },
            )?
        })
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new())
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET])
                .allow_origin(Any),
        );

    info!("Starting server on port 3000");
    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}
