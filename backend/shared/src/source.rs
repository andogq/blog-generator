use std::{fmt::Display, ops::Deref};

use crate::plugin::{AuthPlugin, Plugin};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct SourceIdentifier(String);
impl SourceIdentifier {
    pub fn new(identifier: &str) -> Self {
        Self(identifier.to_string())
    }
}
impl Deref for SourceIdentifier {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Display for SourceIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub trait Source {
    fn get_identifier(&self) -> SourceIdentifier;

    fn get_plugins(&self) -> Vec<Plugin>;
    fn get_auth_plugins(&self) -> Vec<Box<dyn AuthPlugin>>;
}
