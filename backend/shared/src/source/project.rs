use super::IdentifiableSource;

pub trait ProjectsSource: IdentifiableSource + Send + Sync {}
