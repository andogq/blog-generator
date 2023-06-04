use reqwest::{Client, Url};
use serde::Deserialize;

use self::repositories::SearchRepositoriesApi;

mod repositories;

#[derive(Deserialize)]
pub struct SearchResponse<T> {
    total_count: usize,
    incomplete_results: bool,
    items: Vec<T>,
}

pub struct SearchApi {
    pub repositories: SearchRepositoriesApi,
}

impl SearchApi {
    pub fn new(client: &Client, api_base: &Url) -> Self {
        Self {
            repositories: SearchRepositoriesApi::new(client, api_base),
        }
    }
}
