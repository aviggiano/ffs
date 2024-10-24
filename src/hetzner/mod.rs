use super::config;
use hcloud::models::Server;

pub async fn list_servers() -> Result<Vec<Server>, Box<dyn std::error::Error>> {
    use hcloud::apis::configuration::Configuration;
    use hcloud::apis::servers_api;

    let mut configuration = Configuration::new();
    configuration.bearer_access_token =
        Some(config::load_config("./config.toml")?.hcloud_api_token);

    let servers = servers_api::list_servers(&configuration, Default::default())
        .await?
        .servers;

    Ok(servers)
}
