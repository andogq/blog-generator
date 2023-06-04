use serde::Deserialize;
use shared::plugin::UserInformation;

#[derive(Deserialize)]
pub struct UserResponse {
    pub login: String,
    pub avatar_url: String,
    pub html_url: String,
    pub name: Option<String>,
    pub company: Option<String>,
    pub blog: Option<String>,
    pub location: Option<String>,
    pub email: Option<String>,
    pub bio: Option<String>,
    pub twitter_username: Option<String>,
}

impl From<UserResponse> for UserInformation {
    fn from(user: UserResponse) -> Self {
        Self {
            name: user.name,
            avatar: user.avatar_url,
            bio: user.bio,
            location: user.location,
            email: user.email,
            links: [
                ("github".to_string(), Some(user.html_url)),
                (
                    "twitter".to_string(),
                    user.twitter_username
                        .map(|username| format!("https://twitter.com/{username}")),
                ),
            ]
            .into_iter()
            .filter_map(|(site, url)| url.map(|url| (site, url)))
            .collect(),
            blog: user.blog,
            company: user.company,
        }
    }
}
