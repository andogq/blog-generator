mod repositories;
mod user;

pub use repositories::*;
use reqwest::{Client, Url};
pub use user::*;

pub struct RestApi {
    pub repositories: RepositoriesApi,
    pub user: UserApi,
}

impl RestApi {
    pub fn new(client: &Client, api_base: &Url) -> Self {
        Self {
            repositories: RepositoriesApi::new(client, api_base),
            user: UserApi::new(client, api_base),
        }
    }
}
