use std::{
    collections::{BTreeMap, HashMap},
    env, fs,
    io::{Stdout, Write},
    process::{Command, Stdio},
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use toml::Value;

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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PartialWrangler {
    #[serde(flatten)]
    extras: IndexMap<String, Value>,

    vars: HashMap<String, String>,
}

fn main() {
    if let Some(command) = env::args().nth(1) {
        let config = Config::new();

        // Open env file
        println!("Reading env file");
        let env_file = fs::read_to_string(ENV_FILE).expect("Couldn't open env file");

        // Open wrangler file
        println!("Reading wrangler file");
        let original_wrangler_file =
            &fs::read_to_string(WRANGLER_FILE).expect("Couldn't open wrangler.toml file");

        let mut wrangler_file: PartialWrangler =
            toml::from_str(original_wrangler_file).expect("Problem parsing wrangler.toml file");

        match command.as_str() {
            "deploy" => {
                println!("Running deploy");

                for line in env_file
                    .lines()
                    .filter(|line| !(line.starts_with('#') || line.trim().is_empty()))
                {
                    if let Some((key, value)) = line.split_once('=') {
                        if key.starts_with("PUBLIC_") {
                            println!("Found public variable {}", key);

                            // Add to wrangler file
                            wrangler_file
                                .vars
                                .insert(key.to_string(), value.to_string());
                        } else {
                            println!("Found private variable {}", key);

                            if config.upload_secrets {
                                // Run wrangler secret put
                                let mut wrangler_command = Command::new("wrangler")
                                    .args(["-c", WRANGLER_FILE])
                                    .args(["secret", "put"])
                                    .arg(key)
                                    .stdin(Stdio::piped())
                                    .stdout(Stdio::piped())
                                    .stderr(Stdio::piped())
                                    .spawn()
                                    .expect("Problem launching wrangler command");

                                wrangler_command
                                    .stdin
                                    .take()
                                    .expect("Failed to open stdin")
                                    .write_all(value.as_bytes())
                                    .expect("Failed to write to stdin");

                                let output = wrangler_command
                                    .wait_with_output()
                                    .expect("Failed to get output");
                                if !output.status.success() {
                                    println!("Error running `wrangler secret put {}`", key);
                                }
                            }
                        }
                    }
                }

                // Write to wrangler file
                println!("Adding variables to wrangler file");
                fs::write(
                    WRANGLER_FILE,
                    toml::to_string(&wrangler_file).expect("Problem serializing wrangler file"),
                )
                .expect("Problem writing to wrangler file");

                // Publish with wrangler cli
                println!("Publishing with wrangler");
                let wrangler_command = Command::new("wrangler")
                    .current_dir(fs::canonicalize("../worker").expect("path"))
                    .args(["-c", WRANGLER_FILE])
                    .arg("publish")
                    .status()
                    .expect("Problem launching wrangler publish");

                if wrangler_command.success() {
                    println!("Successfully ran wrangler publish");
                } else {
                    println!("Problem running wrangler publish");
                }

                // Restore wrangler file
                println!("Restoring wrangler file");
                fs::write(WRANGLER_FILE, &original_wrangler_file)
                    .expect("Problem writing to wrangler file");
            }
            "dev" => {}
            _ => {}
        }
    }

    println!("Finished! Exiting");
}
