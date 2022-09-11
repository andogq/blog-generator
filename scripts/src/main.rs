use std::{
    collections::HashMap,
    env, fs,
    io::Write,
    process::{Command, Stdio},
};

mod vercel;
use vercel::VercelClient;

const ENV_FILE: &str = "../.env.dev";
const WRANGLER_FILE: &str = "../worker/wrangler.toml";

struct Config {
    pub upload_secrets: bool,
}

impl Config {
    pub fn new() -> Config {
        Config {
            upload_secrets: true,
        }
    }
}

struct Variable {
    pub key: String,
    pub value: String,
    pub public: bool,
}

#[tokio::main]
async fn main() {
    if let Some(command) = env::args().nth(1) {
        let config = Config::new();

        // Open env file
        println!("Reading env file");
        let env_file = fs::read_to_string(ENV_FILE).expect("Couldn't open env file");

        let variables: HashMap<String, Variable> = env_file
            .lines()
            .filter_map(|line| {
                if !(line.starts_with('#') || line.trim().is_empty()) {
                    if let Some((key, value)) = line.split_once('=') {
                        return Some((
                            key.to_string(),
                            Variable {
                                key: key.to_string(),
                                value: value.to_string(),
                                public: key.starts_with("PUBLIC_"),
                            },
                        ));
                    }
                }

                None
            })
            .collect();

        match command.as_str() {
            "deploy" => {
                println!("Running deploy");

                let (vercel_token, vercel_project_id) = (
                    &variables
                        .get("VERCEL_TOKEN")
                        .expect("VERCEL_TOKEN missing from env file")
                        .value,
                    &variables
                        .get("VERCEL_PROJECT_ID")
                        .expect("VERCEL_PROJECT_ID missing from env file")
                        .value,
                );

                // Prepare Vercel client
                let api_client = VercelClient::new(vercel_token, vercel_project_id)
                    .expect("Problem setting up Vercel client");

                let vercel_variables = api_client
                    .list_variables()
                    .await
                    .expect("Problem fetching Vercel variables");

                for variable in variables.values() {
                    // Cloudflare
                    if !variable.public && config.upload_secrets {
                        // Run wrangler secret put
                        let mut wrangler_command = Command::new("wrangler")
                            .args(["-c", WRANGLER_FILE])
                            .args(["secret", "put"])
                            .arg(&variable.key)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .expect("Problem launching wrangler command");

                        wrangler_command
                            .stdin
                            .take()
                            .expect("Failed to open stdin")
                            .write_all(variable.value.as_bytes())
                            .expect("Failed to write to stdin");

                        let output = wrangler_command
                            .wait_with_output()
                            .expect("Failed to get output");
                        if !output.status.success() {
                            println!("Error running `wrangler secret put {}`", &variable.key);
                        }
                    }

                    // Vercel
                    if let Some(matching_variable) =
                        vercel_variables.iter().find(|var| var.key == variable.key)
                    {
                        // Edit existing variable
                        if let Err(e) = api_client
                            .edit_variable(&matching_variable.id, &variable.value)
                            .await
                        {
                            println!(
                                "Problem uploading variable {} to Vercel: {:?}",
                                variable.key, e
                            );
                        }
                    } else {
                        // Create new variable
                        if let Err(e) = api_client
                            .create_variable(&variable.key, &variable.value)
                            .await
                        {
                            println!(
                                "Problem uploading variable {} to Vercel: {:?}",
                                variable.key, e
                            );
                        }
                    }
                }

                // Publish with wrangler CLI
                println!("Publishing with wrangler");
                let wrangler_command = Command::new("wrangler")
                    .current_dir(fs::canonicalize("../worker").expect("path"))
                    .arg("publish")
                    .args(["-c", WRANGLER_FILE])
                    .args(variables.values().filter(|var| var.public).flat_map(|var| {
                        ["--var".to_string(), format!("{}:{}", var.key, var.value)]
                    }))
                    .status()
                    .expect("Problem launching wrangler publish");

                if wrangler_command.success() {
                    println!("Successfully ran wrangler publish");
                } else {
                    println!("Problem running wrangler publish");
                }
            }
            "dev" => {}
            _ => {}
        }
    }

    println!("Finished! Exiting");
}
