use std::{collections::HashMap, path::PathBuf};

use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use worker::{console_log, Fetch, Method, Request, RequestInit};

use crate::utils::Source;

static TARGET: &str = "https://api.github.com";

fn request_init() -> RequestInit {
    let mut init = RequestInit::new();
    init.headers.set("user-agent", "rust").unwrap();
    init.headers
        .set(
            "Authorization",
            "Basic YW5kb2dxOmdocF9iM0ROZGtHdkxZdDZOSjNBdG9FdUp1R1E5cmVTOEw0SU5XRUY=",
        )
        .unwrap();

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
        match response.json().await {
            Ok(object) => Some(object),
            Err(e) => {
                console_log!("{:?}", e);
                None
            }
        }
    } else {
        None
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename = "login")]
    pub username: String,

    pub name: String,
    pub bio: String,

    #[serde(rename = "avatar_url")]
    pub profile_picture: String,

    pub location: String,

    #[serde(rename = "twitter_username")]
    pub twitter: String,

    pub hireable: bool,
    pub company: String,
}

#[derive(Deserialize)]
struct PinnedRepoResponsePinnedItemNode {
    name: String,
}
#[derive(Deserialize)]
struct PinnedRepoResponsePinnedItems {
    nodes: Vec<PinnedRepoResponsePinnedItemNode>,
}
#[derive(Deserialize)]
struct PinnedRepoResponseUser {
    pinnedItems: PinnedRepoResponsePinnedItems,
}
#[derive(Deserialize)]
struct PinnedRepoResponseData {
    user: PinnedRepoResponseUser,
}
#[derive(Deserialize)]
struct PinnedRepoResponse {
    data: PinnedRepoResponseData,
}

impl User {
    pub async fn get(user: &str) -> Option<User> {
        request::<User>(&format!("/users/{}", user)).await
    }

    pub async fn get_repo(&self, repo_name: String) -> Option<Repo> {
        if let (Some(repo), Some(languages)) = (
            request::<Repo>(&format!("/repos/{}/{}", self.username, repo_name)).await,
            request::<HashMap<String, u32>>(&format!(
                "/repos/{}/{}/languages",
                self.username, repo_name
            ))
            .await,
        ) {
            Some(Repo {
                languages: languages.keys().into_iter().cloned().collect(),
                ..repo
            })
        } else {
            None
        }
    }

    pub async fn get_pinned(&self, gh_key: &str) -> Vec<Repo> {
        let mut init = request_init();
        init.method = Method::Post;
        init.headers
            .set(
                "Authorization",
                &format!(
                    "Basic {}",
                    base64::encode(format!("{}:{}", "andogq", gh_key))
                ),
            )
            .unwrap();
        init.body = Some(format!(
            r#"{{
	"query": "query{{user(login:\"{}\"){{pinnedItems(first:6,types:REPOSITORY){{nodes{{... on Repository{{name}}}}}}}}}}",
	"variables": {{}}
}}"#, self.username)
            .into(),
        );

        let endpoint = format!("{}{}", TARGET, "/graphql");
        let request = Fetch::Request(Request::new_with_init(&endpoint, &init).unwrap());

        if let Ok(mut response) = request.send().await {
            console_log!("{} {}", endpoint, response.status_code());
            if let Ok(object) = response.json::<PinnedRepoResponse>().await {
                futures::future::join_all(
                    object
                        .data
                        .user
                        .pinnedItems
                        .nodes
                        .into_iter()
                        .map(|repo| self.get_repo(repo.name)),
                )
                .await
                .into_iter()
                .flatten()
                .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
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
    pub archived: bool,

    pub created_at: String,
    pub updated_at: String,
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
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileContent {
    content: String,
    encoding: String,
}

impl File {
    pub fn new(path: &str) -> File {
        File {
            name: PathBuf::from(path)
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .to_string(),
            path: path.to_string(),
        }
    }

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

#[derive(Deserialize)]
pub struct Repo {
    pub full_name: String,
    pub name: String,

    pub homepage: Option<String>,
    pub description: Option<String>,
    pub topics: Vec<String>,

    // Field doesn't come from repo API call, second call must happen
    #[serde(default)]
    pub languages: Vec<String>,

    #[serde(rename = "forks_count")]
    pub forks: u32,
    #[serde(rename = "stargazers_count")]
    pub stargazers: u32,
    #[serde(rename = "watchers_count")]
    pub watchers: u32,

    pub html_url: String,
}

impl Repo {
    pub async fn get_issue(&self, issue_number: u32) -> Option<Issue> {
        request::<Issue>(&format!(
            "/repos/{}/issues/{}",
            self.full_name, issue_number
        ))
        .await
    }

    pub async fn get_issues(&self) -> Vec<Issue> {
        request::<Vec<Issue>>(&format!("/repos/{}/issues", self.full_name))
            .await
            .unwrap_or_default()
    }

    pub fn get_contents_path(&self, path: &str) -> String {
        format!("/repos/{}/contents{}", self.full_name, path)
    }
}
