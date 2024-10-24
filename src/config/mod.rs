use serde::Deserialize;
use std::fs;
use toml::Value;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub hcloud_api_token: String,
}

pub fn load_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(file_path)?;
    let value = contents.parse::<Value>()?;
    let hcloud_api_token = value["hcloud_token"]
        .as_str()
        .ok_or("hcloud_token not found or not a string")?
        .to_string();

    Ok(Config { hcloud_api_token })
}
