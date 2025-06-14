use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;

use async_trait::async_trait;
use hcloud::apis::configuration::Configuration;
use hcloud::apis::servers_api;
use hcloud::apis::servers_api::{CreateServerParams, DeleteServerParams, ListServersParams};
use hcloud::models::CreateServerRequest;
use ssh2::Session;

use super::Provider;
use crate::config::Config;
use crate::jobs::Job;

#[async_trait]
impl Provider for HetznerProvider {
    async fn start_job(&self, name: &str) -> Result<Job, Box<dyn std::error::Error + Send + Sync>> {
        let config = Config::new();
        let mut configuration = Configuration::new();
        configuration.bearer_access_token = Some(config.hcloud_api_token);

        let params = CreateServerParams {
            create_server_request: Some(CreateServerRequest {
                name: name.to_string(),
                image: config.image,
                server_type: config.server_type,
                location: Some(config.location),
                ssh_keys: Some(vec![config.ssh_key_name]),
                user_data: Some(config.user_data),
                ..Default::default()
            }),
        };
        let res = servers_api::create_server(&configuration, params).await?;

        let job = Job {
            id: res.server.id.to_string(),
            ipv4: res.server.public_net.ipv4.unwrap().ip,
            name: Some(name.to_string()),
        };

        let ip = job.ipv4.clone();
        let key_path = config.ssh_key_path.clone();
        tokio::spawn(async move {
            let _ =
                tokio::task::spawn_blocking(move || super::install_over_ssh(&ip, &key_path)).await;
        });

        Ok(job)
    }

    async fn get_job(
        &self,
        id: &str,
    ) -> Result<Option<Job>, Box<dyn std::error::Error + Send + Sync>> {
        let config = Config::new();
        let mut configuration = Configuration::new();
        configuration.bearer_access_token = Some(config.hcloud_api_token);

        let server = servers_api::get_server(
            &configuration,
            hcloud::apis::servers_api::GetServerParams {
                id: id.parse::<i64>().unwrap(),
            },
        )
        .await?
        .server;

        server.map_or_else(
            || Ok(None),
            |server| {
                Ok(Some(Job {
                    id: server.id.to_string(),
                    ipv4: server.public_net.ipv4.unwrap().ip,
                    name: Some(server.name),
                }))
            },
        )
    }

    async fn stop_job(
        &self,
        job_id: &str,
    ) -> Result<Job, Box<dyn std::error::Error + Send + Sync>> {
        let config = Config::new();
        let mut configuration = Configuration::new();
        configuration.bearer_access_token = Some(config.hcloud_api_token);
        let params = DeleteServerParams {
            id: job_id.parse::<i64>().unwrap(),
        };
        servers_api::delete_server(&configuration, params).await?;

        Ok(Job {
            id: job_id.to_string(),
            ipv4: String::new(),
            name: None,
        })
    }

    async fn list_jobs(&self) -> Result<Vec<Job>, Box<dyn std::error::Error + Send + Sync>> {
        let config = Config::new();
        let mut configuration = Configuration::new();
        configuration.bearer_access_token = Some(config.hcloud_api_token);

        let servers = servers_api::list_servers(&configuration, ListServersParams::default())
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
        follow: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let config = Config::new();
        let mut configuration = Configuration::new();
        configuration.bearer_access_token = Some(config.hcloud_api_token);

        let job = self.get_job(id).await?;

        let ipv4 = job.unwrap().ipv4;
        let tcp = TcpStream::connect((ipv4.as_str(), 22))?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;

        // Authenticate using a private key
        sess.userauth_pubkey_file("root", None, Path::new(&config.ssh_key_path), None)?;

        // Open a channel
        let mut channel = sess.channel_session()?;

        // Determine the command based on follow flag
        let command = if follow {
            format!("tail -f {}", super::DEFAULT_LOG_FILE)
        } else {
            format!("cat {}", super::DEFAULT_LOG_FILE)
        };

        channel.exec(&command)?;

        if follow {
            let mut buffer = [0; 1024];
            loop {
                match channel.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        let output = String::from_utf8_lossy(&buffer[..n]);
                        print!("{}", output);
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            continue;
                        }
                        return Err(Box::new(e));
                    }
                }
            }
        } else {
            let mut s = String::new();
            channel.read_to_string(&mut s)?;
            print!("{s}");
        }

        channel.wait_close()?;

        Ok(())
    }

    async fn scp(
        &self,
        id: &str,
        filename: &str,
        destination: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let config = Config::new();
        let mut configuration = Configuration::new();
        configuration.bearer_access_token = Some(config.hcloud_api_token);

        let job = self.get_job(id).await?;

        let ipv4 = job.unwrap().ipv4;
        let tcp = TcpStream::connect((ipv4.as_str(), 22))?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;

        let mut channel = sess.channel_session()?;
        channel.exec(&format!("scp -r root@{ipv4}:{filename} {destination}"))?;

        // Read and print output in real-time
        let mut buffer = [0; 1024];
        loop {
            match channel.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let output = String::from_utf8_lossy(&buffer[..n]);
                    print!("{output}");
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        continue;
                    }
                    return Err(Box::new(e));
                }
            }
        }

        channel.wait_close()?;
        println!("{}", channel.exit_status()?);

        Ok(())
    }
}

#[derive(Clone)]
pub struct HetznerProvider {}

impl Default for HetznerProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl HetznerProvider {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}
