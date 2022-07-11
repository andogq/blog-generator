use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use worker::{console_log, Fetch, Request, RequestInit};

use crate::utils::Source;

async fn request<T>(endpoint: &str) -> Option<T>
where
    T: DeserializeOwned,
{
    let mut init = RequestInit::new();
    init.headers.set("user-agent", "rust").unwrap();

    let request = Fetch::Request(Request::new_with_init(endpoint, &init).unwrap());

    if let Ok(mut response) = request.send().await {
        console_log!("{} {}", endpoint, response.status_code());
        if let Ok(object) = response.json().await {
            Some(object)
        } else {
            None
        }
    } else {
        None
    }
}

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

impl User {
    pub async fn get(user: &str) -> Option<User> {
        request::<User>(&format!("https://api.github.com/users/{}", user)).await
    }
}

impl Source for User {
    fn get_key_values(&self) -> Vec<(String, String)> {
        vec![
            ("username".to_string(), self.username.to_string()),
            ("name".to_string(), self.name.to_string()),
            ("bio".to_string(), self.bio.to_string()),
            (
                "profile_picture".to_string(),
                self.profile_picture.to_string(),
            ),
            ("location".to_string(), self.location.to_string()),
            ("twitter".to_string(), self.twitter.to_string()),
            ("hireable".to_string(), self.hireable.to_string()),
            ("company".to_string(), self.company.to_string()),
        ]
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Issue {
    #[serde(rename = "number")]
    id: u32,
    title: String,
    pub body: String,

    #[serde(rename = "state", deserialize_with = "deserialize_archive")]
    archived: bool,

    created_at: String,
    updated_at: String,
}

fn deserialize_archive<'de, D>(deserialize: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let v = String::deserialize(deserialize)?;

    // If it is 'closed' then it is archived
    Ok(v.eq("closed"))
}

pub struct Repo {
    pub name: String,
}

impl Repo {
    pub async fn get_issues(&self) -> Vec<Issue> {
        request::<Vec<Issue>>(&format!(
            "https://api.github.com/repos/{}/issues",
            self.name
        ))
        .await
        .unwrap_or_default()
    }
}
