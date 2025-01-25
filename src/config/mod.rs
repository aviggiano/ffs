use std::fs;

use serde::Deserialize;
use toml::Value;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub hcloud_api_token: String,
    pub ssh_key_path: String,
    pub ssh_key_name: String,
    pub image: String,
    pub server_type: String,
    pub location: String,
}

#[derive(Debug)]
struct ConfigError(String);

impl std::error::Error for ConfigError {}
impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn load_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error + Send + Sync>> {
    let contents = fs::read_to_string(file_path)?;
    let value = contents.parse::<Value>()?;
    let hcloud_api_token = value["hcloud_token"]
        .as_str()
        .ok_or(ConfigError(
            "hcloud_token not found or not a string".to_string(),
        ))?
        .to_string();
    let ssh_key_path = value["ssh_key_path"]
        .as_str()
        .ok_or(ConfigError("ssh_key not found or not a string".to_string()))?
        .to_string();
    let ssh_key_name = value["ssh_key_name"]
        .as_str()
        .ok_or(ConfigError(
            "ssh_key_name not found or not a string".to_string(),
        ))?
        .to_string();
    let image = value["image"]
        .as_str()
        .ok_or(ConfigError("image not found or not a string".to_string()))?
        .to_string();
    let server_type = value["server_type"]
        .as_str()
        .ok_or(ConfigError(
            "server_type not found or not a string".to_string(),
        ))?
        .to_string();
    let location = value["location"]
        .as_str()
        .ok_or(ConfigError(
            "location not found or not a string".to_string(),
        ))?
        .to_string();

    Ok(Config {
        hcloud_api_token,
        ssh_key_path,
        ssh_key_name,
        image,
        server_type,
        location,
    })
}
