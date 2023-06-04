mod repositories;
mod user;

pub use repositories::*;
pub use user::*;

static API_BASE: &str = "https://api.github.com/";
