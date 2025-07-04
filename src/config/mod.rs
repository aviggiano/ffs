use serde::Deserialize;

use crate::database::Database;
#[derive(Debug, Deserialize)]
pub struct Config {
    pub hcloud_api_token: String,
    pub ssh_key_path: String,
    pub ssh_key_name: String,
    pub image: String,
    pub server_type: String,
    pub location: String,
    pub user_data: String,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    #[must_use]
    pub fn new() -> Self {
        let database = Database::new();
        let hcloud_api_token = database.get("hcloud_token").unwrap_or_default();
        let ssh_key_path = database.get("ssh_key_path").unwrap_or_default();
        let ssh_key_name = database.get("ssh_key_name").unwrap_or_default();
        let image = database.get("image").unwrap_or_default();
        let server_type = database.get("server_type").unwrap_or_default();
        let location = database.get("location").unwrap_or_default();
        let user_data = database.get("user_data").unwrap_or_default();

        Self {
            hcloud_api_token,
            ssh_key_path,
            ssh_key_name,
            image,
            server_type,
            location,
            user_data,
        }
    }
}

/// Loads configuration from a config file path.
///
/// # Errors
///
/// This function will return an error if the config cannot be loaded.
pub fn load_config(_config_path: &str) -> Result<Config, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Config::new())
}
