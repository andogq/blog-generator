use url::{ParseError, Url};

use super::Scope;

pub fn generate_redirect_url(
    api_base: &Url,
    client_id: &str,
    scopes: &[Scope],
    redirect_url: &str,
) -> Result<Url, ParseError> {
    let mut url = api_base.join("authorize")?;
    url.query_pairs_mut().extend_pairs([
        (
            "scope",
            scopes
                .iter()
                .map(String::from)
                .collect::<Vec<_>>()
                .join(" ")
                .as_str(),
        ),
        ("client_id", client_id),
        ("redirect_uri", redirect_url),
    ]);

    Ok(url)
}
