use std::{collections::HashMap, time::Duration};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use worker::*;

pub struct CloudflareAPI {
    zone_id: String,
    target: String,
    token: String,
}

#[derive(Deserialize, Clone)]
struct CloudflareResponse<T> {
    success: bool,
    #[serde(default)]
    errors: Vec<CloudflareMessage>,
    #[serde(default)]
    messages: Vec<CloudflareMessage>,
    result: Option<T>,
}

#[derive(Deserialize, Debug, Clone)]
struct CloudflareMessage {
    message: String,
    code: Option<usize>,
}

#[derive(Deserialize, Clone)]
struct CloudflareCustomHostnameValidationRecord {
    txt_name: Option<String>,
    txt_value: Option<String>,
    http_url: Option<String>,
    http_body: Option<String>,
    #[serde(default)]
    emails: Vec<String>,
}

#[derive(Deserialize, Clone)]
struct CloudflareCustomHostnameSettings {
    http2: Option<String>,
    min_tls_version: Option<String>,
    tls_1_3: Option<String>,
    #[serde(default)]
    ciphers: Vec<String>,
    early_hints: Option<String>,
}

#[derive(Deserialize, Clone)]
struct CloudflareCustomHostnameSSL {
    id: String,
    status: String,
    method: String,
    #[serde(rename = "type")]
    _type: String,
    #[serde(default)]
    validation_records: Vec<CloudflareCustomHostnameValidationRecord>,
    #[serde(default)]
    validation_errors: Vec<CloudflareMessage>,
    #[serde(default)]
    hosts: Vec<String>,
    issuer: Option<String>,
    serial_number: Option<String>,
    signature: Option<String>,
    uploaded_on: Option<String>,
    expires_on: Option<String>,
    custom_csr_id: Option<String>,
    settings: Option<CloudflareCustomHostnameSettings>,
    bundle_method: Option<String>,
    wildcard: bool,
    certificate_authority: Option<String>,
    custom_certificate: Option<String>,
    custom_key: Option<String>,
}

#[derive(Deserialize, Clone)]
struct CloudflareCustomHostnameOwnershipVerification {
    #[serde(rename = "type")]
    _type: String,
    name: String,
    value: String,
}

#[derive(Deserialize, Clone)]
struct CloudflareCustomHostnameOwnershipVerificationHttp {
    http_url: String,
    http_body: String,
}

#[derive(Deserialize, Clone)]
struct CloudflareCustomHostname {
    id: String,
    hostname: String,
    ssl: Option<CloudflareCustomHostnameSSL>,
    #[serde(default)]
    custom_metadata: HashMap<String, String>,
    custom_origin_server: Option<String>,
    custom_origin_sni: Option<String>,
    status: String,
    #[serde(default)]
    verification_errors: Vec<String>,
    ownership_verification: Option<CloudflareCustomHostnameOwnershipVerification>,
    ownership_verification_http: Option<CloudflareCustomHostnameOwnershipVerificationHttp>,
    created_at: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct Record {
    record_type: String,
    name: String,
    value: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct CustomHostname {
    id: String,
    hostname: String,
    dns_records: Vec<Record>,
    verification_status: String,
    ssl_status: Option<String>,
    errors: Vec<String>,
}

impl From<CloudflareCustomHostname> for CustomHostname {
    fn from(custom_hostname: CloudflareCustomHostname) -> Self {
        CustomHostname {
            id: custom_hostname.id,
            hostname: custom_hostname.hostname,
            dns_records: {
                let mut records: Vec<Record> = custom_hostname
                    .ssl
                    .as_ref()
                    .map(|ssl| {
                        ssl.validation_records
                            .iter()
                            .filter_map(|record| {
                                if let (Some(name), Some(value)) =
                                    (record.txt_name.clone(), record.txt_value.clone())
                                {
                                    Some(Record {
                                        name,
                                        value,
                                        record_type: "txt".to_string(),
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                if let Some(ownership_record) = custom_hostname.ownership_verification {
                    records.push(Record {
                        name: ownership_record.name,
                        value: ownership_record.value,
                        record_type: ownership_record._type,
                    });
                }

                records
            },
            verification_status: custom_hostname.status,
            ssl_status: custom_hostname.ssl.as_ref().map(|ssl| ssl.status.clone()),
            errors: [
                custom_hostname.verification_errors,
                custom_hostname
                    .ssl
                    .map(|ssl| {
                        ssl.validation_errors
                            .iter()
                            .map(|e| e.message.clone())
                            .collect()
                    })
                    .unwrap_or_default(),
            ]
            .concat(),
        }
    }
}

impl CloudflareAPI {
    pub fn new(zone_id: &str, target: &str, token: &str) -> CloudflareAPI {
        CloudflareAPI {
            zone_id: zone_id.to_string(),
            target: target.to_string(),
            token: token.to_string(),
        }
    }

    pub async fn route(
        &self,
        mut path: Box<dyn Iterator<Item = String> + '_>,
        request: Request,
    ) -> Result<Response> {
        match (request.method(), path.next().as_deref()) {
            (Method::Get, Some("hostnames")) => Response::from_json(&self.get_hostnames().await),
            (Method::Get, Some("hostname")) => {
                if let Some(hostname) = path.next() {
                    if let Some(hostname) = self.get_hostname(&hostname).await {
                        Response::from_json(&hostname)
                    } else {
                        Response::error("Not found", 404)
                    }
                } else {
                    Response::error("Problem with request", 400)
                }
            }
            (Method::Post, Some("hostname")) => {
                if let Some(hostname) = path.next() {
                    if let Some(hostname) = self.add_hostname(&hostname).await {
                        Response::from_json(&hostname)
                    } else {
                        Response::error("Problem creating hostname", 500)
                    }
                } else {
                    Response::error("Invalid request", 400)
                }
            }
            _ => Response::error("Not found", 404),
        }
    }

    async fn get_hostnames(&self) -> Vec<CustomHostname> {
        self.fetch::<Vec<CloudflareCustomHostname>>("/custom_hostnames", Method::Get, None)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|custom_hostname| custom_hostname.into())
            .collect()
    }

    async fn get_hostname(&self, id: &str) -> Option<CustomHostname> {
        self.fetch::<CloudflareCustomHostname>(
            &format!("/custom_hostnames/{}", id),
            Method::Get,
            None,
        )
        .await
        .map(|custom_hostname| custom_hostname.into())
    }

    async fn add_hostname(&self, hostname: &str) -> Option<CustomHostname> {
        if let Some(custom_hostname) = self
            .fetch::<CloudflareCustomHostname>(
                "/custom_hostnames",
                Method::Post,
                Some(json!({
                    "hostname": hostname,
                    "ssl": json!({
                        "method": "txt",
                        "type": "dv"
                    })
                })),
            )
            .await
        {
            let mut ch = None;
            let mut counter = 1;

            // Poll for ssl_status pending, so that DNS TXT record will be presented
            while ch.is_none() && counter < 10 {
                Delay::from(Duration::from_millis(1000 * counter)).await;

                if let Some(custom_hostname) = self.get_hostname(&custom_hostname.id).await {
                    if custom_hostname
                        .ssl_status
                        .as_ref()
                        .map(|status| status == "pending_validation")
                        .unwrap_or(false)
                    {
                        ch = Some(custom_hostname);
                    }
                }

                counter += 1;
            }

            ch
        } else {
            None
        }
    }

    async fn fetch<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        method: Method,
        body: Option<serde_json::Value>,
    ) -> Option<T> {
        if let Ok(request) = {
            let mut init = RequestInit::new();

            init.with_method(method);
            if let Some(body) = body {
                init.with_body(Some(body.to_string().into()));
            }

            init.with_headers({
                let mut headers = Headers::new();
                headers
                    .set("Authorization", &format!("Bearer {}", self.token))
                    .unwrap();
                headers.set("Content-Type", "application/json").unwrap();
                headers
            });

            Request::new_with_init(
                &format!("{}/zones/{}{}", self.target, self.zone_id, endpoint),
                &init,
            )
        } {
            if let Ok(mut response) = Fetch::Request(request).send().await {
                console_log!("Status code: {}", response.status_code());
                match response.json::<CloudflareResponse<T>>().await {
                    Ok(body) => {
                        if body.success {
                            return body.result;
                        } else {
                            console_error!("Cloudflare error: {:?}", body.errors);
                        }
                    }
                    Err(e) => console_error!("Deserializing error: {:?}", e),
                }
            }
        }

        None
    }
}
