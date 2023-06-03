mod access_token;
mod redirect;
mod scope;

pub use access_token::*;
pub use redirect::*;
pub use scope::*;

static API_BASE: &str = "https://github.com/login/oauth/";
