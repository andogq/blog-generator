use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, Method, StatusCode,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

const TARGET: &str = "https://api.vercel.com";

pub struct VercelClient {
    project_id: String,
    client: Client,
}

#[derive(Debug)]
pub enum VercelError {
    Initialisation,
    Non200(StatusCode),
    Request,
    Deserialization(reqwest::Error),
    Serialization(serde_json::Error),
}

#[derive(Deserialize, Debug)]
pub struct EnvVariable {
    pub id: String,
    pub key: String,
}

#[derive(Deserialize)]
struct EnvList {
    pub envs: Vec<EnvVariable>,
}

#[derive(Serialize)]
struct EnvVariableValue {
    pub value: String,
}

#[derive(Serialize)]
struct EnvVariableNew {
    pub key: String,
    pub value: String,
    pub target: Vec<String>,
    #[serde(rename = "type")]
    pub _type: String,
}

impl EnvVariableNew {
    pub fn new(key: &str, value: &str) -> EnvVariableNew {
        EnvVariableNew {
            key: key.to_string(),
            value: value.to_string(),
            target: vec![
                "production".to_string(),
                "preview".to_string(),
                "development".to_string(),
            ],
            _type: "encrypted".to_string(),
        }
    }
}

struct RequestBuilder {
    request_builder: reqwest::RequestBuilder,
}

impl RequestBuilder {
    pub fn body<T>(self, body: T) -> Result<RequestBuilder, VercelError>
    where
        T: Serialize,
    {
        Ok(RequestBuilder {
            request_builder: self
                .request_builder
                .body(serde_json::to_string(&body).map_err(VercelError::Serialization)?)
                .header("Content-Type", "application/json"),
        })
    }

    pub async fn send(self) -> Result<reqwest::Response, VercelError> {
        self.request_builder
            .send()
            .await
            .map_err(|_| VercelError::Request)
    }
}

impl VercelClient {
    pub fn new(token: &str, project_id: &str) -> Result<VercelClient, VercelError> {
        Ok(VercelClient {
            project_id: project_id.to_string(),
            client: Client::builder()
                .default_headers({
                    let mut headers = HeaderMap::new();
                    headers.insert(
                        "Authorization",
                        HeaderValue::from_str(&format!("Bearer {}", token))
                            .expect("Problem making authorization header"),
                    );

                    headers
                })
                .build()
                .map_err(|_| VercelError::Initialisation)?,
        })
    }

    fn request_builder(&self, url: &str, method: Method) -> RequestBuilder {
        RequestBuilder {
            request_builder: self.client.request(
                method,
                format!("{}/v9/projects/{}{}", TARGET, self.project_id, url),
            ),
        }
    }

    async fn request<T>(&self, url: &str, method: Method) -> Result<T, VercelError>
    where
        T: DeserializeOwned,
    {
        let response = self.request_builder(url, method).send().await?;

        match response.status() {
            StatusCode::OK => response
                .json::<T>()
                .await
                .map_err(VercelError::Deserialization),
            status_code => Err(VercelError::Non200(status_code)),
        }
    }

    pub async fn list_variables(&self) -> Result<Vec<EnvVariable>, VercelError> {
        Ok(self.request::<EnvList>("/env", Method::GET).await?.envs)
    }

    pub async fn edit_variable(&self, id: &str, value: &str) -> Result<(), VercelError> {
        let response = self
            .request_builder(&format!("/env/{}", id), Method::PATCH)
            .body(EnvVariableValue {
                value: value.to_string(),
            })?
            .send()
            .await?;

        if response.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(VercelError::Non200(response.status()))
        }
    }

    pub async fn create_variable(&self, key: &str, value: &str) -> Result<(), VercelError> {
        let response = self
            .request_builder("/env", Method::POST)
            .body(EnvVariableNew::new(key, value))?
            .send()
            .await?;

        if response.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(VercelError::Non200(response.status()))
        }
    }
}
