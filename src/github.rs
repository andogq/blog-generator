use serde::{Deserialize, Serialize};
use worker::{Fetch, Request, RequestInit};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename = "login")]
    username: String,

    name: String,
    bio: String,

    #[serde(rename = "avatar_url")]
    profile_picture: String,

    location: String,

    #[serde(rename = "twitter_username")]
    twitter: String,

    hireable: bool,
    company: String,
}

pub async fn get_user(user: &str) -> Option<User> {
    let mut init = RequestInit::new();
    init.headers.set("user-agent", "rust").unwrap();

    let request = Fetch::Request(
        Request::new_with_init("https://api.github.com/users/andogq", &init).unwrap(),
    );

    if let Ok(mut response) = request.send().await {
        if let Ok(user) = response.json().await {
            Some(user)
        } else {
            None
        }
    } else {
        None
    }
}
