use crate::plugin::PluginCollection;

pub trait Source {
    fn get_plugins(&self) -> PluginCollection;
}
