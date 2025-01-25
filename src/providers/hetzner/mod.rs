use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;

use async_trait::async_trait;
use hcloud::apis::configuration::Configuration;
use hcloud::apis::servers_api;
use hcloud::apis::servers_api::{CreateServerParams, DeleteServerParams};
use hcloud::models::CreateServerRequest;
use ssh2::Session;

use super::super::config;
use super::super::config::Config;
use super::Provider;
use crate::jobs::Job;

#[async_trait]
impl Provider for HetznerProvider {
    async fn start_job(&self, name: &str) -> Result<Job, Box<dyn std::error::Error + Send + Sync>> {
        let config = config()?;
        let mut configuration = Configuration::new();
        configuration.bearer_access_token = Some(config.hcloud_api_token);

        let params = CreateServerParams {
            create_server_request: Some(CreateServerRequest {
                name: name.to_string(),
                image: config.image,
                server_type: config.server_type,
                location: Some(config.location),
                ssh_keys: Some(vec![config.ssh_key_name]),
                ..Default::default()
            }),
        };
        let res = servers_api::create_server(&configuration, params).await?;

        let job = Job {
            id: res.server.id.to_string(),
            ipv4: res.server.public_net.ipv4.unwrap().ip,
            name: Some(name.to_string()),
        };

        Ok(job)
    }

    async fn get_job(
        &self,
        id: &str,
    ) -> Result<Option<Job>, Box<dyn std::error::Error + Send + Sync>> {
        let mut configuration = Configuration::new();
        configuration.bearer_access_token = Some(config()?.hcloud_api_token);

        let server = servers_api::get_server(
            &configuration,
            hcloud::apis::servers_api::GetServerParams {
                id: id.parse::<i64>().unwrap(),
            },
        )
        .await?
        .server;

        match server {
            Some(server) => Ok(Some(Job {
                id: server.id.to_string(),
                ipv4: server.public_net.ipv4.unwrap().ip,
                name: Some(server.name),
            })),
            None => Ok(None),
        }
    }

    async fn stop_job(
        &self,
        job_id: &str,
    ) -> Result<Job, Box<dyn std::error::Error + Send + Sync>> {
        let config = config()?;
        let mut configuration = Configuration::new();
        configuration.bearer_access_token = Some(config.hcloud_api_token);
        let params = DeleteServerParams {
            id: job_id.parse::<i64>().unwrap(),
        };
        servers_api::delete_server(&configuration, params).await?;

        Ok(Job {
            id: job_id.to_string(),
            ipv4: "".to_string(),
            name: None,
        })
    }

    async fn list_jobs(&self) -> Result<Vec<Job>, Box<dyn std::error::Error + Send + Sync>> {
        let mut configuration = Configuration::new();
        configuration.bearer_access_token = Some(config()?.hcloud_api_token);

        let servers = servers_api::list_servers(&configuration, Default::default())
            .await?
            .servers;

        let jobs = servers
            .into_iter()
            .map(|server| Job {
                id: server.id.to_string(),
                ipv4: server.public_net.ipv4.unwrap().ip,
                name: Some(server.name),
            })
            .collect();

        Ok(jobs)
    }

    async fn tail(
        &self,
        id: &str,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let config = config()?;
        let mut configuration = Configuration::new();
        configuration.bearer_access_token = Some(config.hcloud_api_token);

        let job = self.get_job(id).await?;

        let ipv4 = job.unwrap().ipv4;
        let tcp = TcpStream::connect((ipv4, 22))?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;

        // Authenticate using a private key
        sess.userauth_pubkey_file("root", None, Path::new(&config.ssh_key_path), None)?;

        // Open a channel
        let mut channel = sess.channel_session()?;

        // Execute command to read log file
        channel.exec(&format!("cat {}", &filename))?;

        // Read the output
        let mut s = String::new();
        channel.read_to_string(&mut s)?;

        // Print the logs
        println!("{s}");

        // Close the channel
        channel.wait_close()?;
        println!("{}", channel.exit_status()?);

        Ok(())
    }
}

fn config() -> Result<Config, Box<dyn std::error::Error + Send + Sync>> {
    config::load_config("./config.toml")
}
pub struct HetznerProvider {}

impl HetznerProvider {
    pub fn new() -> Self {
        Self {}
    }
}
