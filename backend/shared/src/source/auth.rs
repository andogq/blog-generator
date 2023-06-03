use axum::Router;
use std::ops::Deref;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct AuthIdentifier(String);
impl Deref for AuthIdentifier {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AuthIdentifier {
    pub fn new(identifier: &str) -> Self {
        Self(identifier.to_string())
    }
}

pub trait AuthSource {
    fn get_identifier(&self) -> AuthIdentifier;
    fn register_routes(
        &self,
        user_agent: &str,
        save_auth_token: UnboundedSender<(AuthIdentifier, String, String)>,
    ) -> Router;
}
