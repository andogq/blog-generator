use crate::plugin::{AuthPlugin, Plugin};

pub trait Source {
    fn get_plugins(&self) -> Vec<(String, Plugin)>;
    fn get_auth_plugins(&self) -> Vec<(String, Box<dyn AuthPlugin>)>;
}
