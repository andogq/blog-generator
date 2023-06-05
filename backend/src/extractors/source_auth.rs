use axum::{
    async_trait,
    extract::{FromRequestParts, Path},
    http::request::Parts,
    Extension, RequestPartsExt,
};
use reqwest::StatusCode;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::Deserialize;
use std::sync::Arc;

use entities::prelude::UserSource;

pub struct SourceAuth {
    pub source: String,
    pub username: String,
    pub token: String,
}

#[derive(Deserialize)]
struct QueryParams {
    username: String,
    source_identifier: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for SourceAuth {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let db = parts
            .extract::<Extension<Arc<DatabaseConnection>>>()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .0;
        let params = parts
            .extract::<Path<QueryParams>>()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .0;

        UserSource::find_by_id((params.username, params.source_identifier))
            .one(db.as_ref())
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            .and_then(|user_source| user_source.ok_or(StatusCode::UNAUTHORIZED))
            .map(|user_source| SourceAuth {
                source: user_source.site,
                username: user_source.username,
                token: user_source.token,
            })
    }
}
