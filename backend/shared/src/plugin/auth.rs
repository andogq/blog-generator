use axum::Router;
use tokio::sync::mpsc::UnboundedSender;

pub trait AuthPlugin {
    fn register_routes(
        &self,
        source_identifier: &str,
        save_auth_token: UnboundedSender<(String, String, String)>,
    ) -> Router;
}
