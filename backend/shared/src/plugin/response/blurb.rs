use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BlurbResponse {
    blurb: String,
}

impl From<String> for BlurbResponse {
    fn from(s: String) -> Self {
        Self { blurb: s }
    }
}
