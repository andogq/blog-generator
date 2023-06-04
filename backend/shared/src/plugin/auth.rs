use axum::Router;

use super::SaveAuthToken;

pub trait AuthPlugin {
    fn register_routes(&self, source_identifier: &str, save_auth_token: SaveAuthToken) -> Router;
}
