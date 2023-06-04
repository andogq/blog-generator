mod repositories;
mod search;
mod user;

use reqwest::{Client, Url};

pub use repositories::*;
pub use search::*;
pub use user::*;

pub struct RestApi {
    pub repositories: RepositoriesApi,
    pub user: UserApi,
    pub search: SearchApi,
}

impl RestApi {
    pub fn new(client: &Client, api_base: &Url) -> Self {
        Self {
            repositories: RepositoriesApi::new(client, api_base),
            user: UserApi::new(client, api_base),
            search: SearchApi::new(client, api_base),
        }
    }
}
