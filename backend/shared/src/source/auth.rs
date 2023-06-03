use axum::Router;
use tokio::sync::mpsc::UnboundedSender;

use super::IdentifiableSource;

pub trait AuthSource: IdentifiableSource {
    fn register_routes(
        &self,
        user_agent: &str,
        save_auth_token: UnboundedSender<(String, String, String)>,
    ) -> Router;
}
