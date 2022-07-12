use std::collections::HashMap;

use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use worker::{console_log, Fetch, Request, RequestInit};

use crate::utils::Source;

static TARGET: &str = "https://api.github.com";

fn request_init() -> RequestInit {
    let mut init = RequestInit::new();
    init.headers.set("user-agent", "rust").unwrap();

    init
}

async fn request<T>(endpoint: &str) -> Option<T>
where
    T: DeserializeOwned,
{
    let init = request_init();

    let endpoint = format!("{}{}", TARGET, endpoint);
    let request = Fetch::Request(Request::new_with_init(&endpoint, &init).unwrap());

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

    pub name: String,
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
        request::<User>(&format!("/users/{}", user)).await
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
    pub id: u32,
    pub title: String,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub name: String,
    path: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileContent {
    content: String,
    encoding: String,
}

impl File {
    pub async fn get_contents(&self) -> Option<String> {
        if let Some(file) = request::<FileContent>(&self.path).await {
            if file.encoding == "base64" {
                let encoded_content = file.content.replace('\n', "");

                if let Ok(Ok(content)) = base64::decode(encoded_content).map(String::from_utf8) {
                    Some(content)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct Repo {
    pub user: String,
    pub name: String,
}

impl Repo {
    pub async fn get_issue(&self, issue_number: u32) -> Option<Issue> {
        request::<Issue>(&format!(
            "/repos/{}/{}/issues/{}",
            self.user, self.name, issue_number
        ))
        .await
    }

    pub async fn get_issues(&self) -> Vec<Issue> {
        request::<Vec<Issue>>(&format!("/repos/{}/{}/issues", self.user, self.name))
            .await
            .unwrap_or_default()
    }

    pub async fn get_files(&self, path: &str) -> HashMap<String, File> {
        let path = format!("/repos/{}/{}/contents{}", self.user, self.name, path);

        request::<Vec<File>>(&path)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|file| {
                (
                    file.name.clone(),
                    File {
                        path: format!("{}/{}", path, file.name),
                        ..file
                    },
                )
            })
            .collect()
    }
}
