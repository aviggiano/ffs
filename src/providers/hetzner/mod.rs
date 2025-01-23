use std::error::Error;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;

use hcloud::apis::configuration::Configuration;
use hcloud::apis::servers_api;
use hcloud::apis::servers_api::{CreateServerParams, DeleteServerParams};
use hcloud::models::CreateServerRequest;
use ssh2::Session;

use super::super::config;
use super::super::config::Config;
use super::super::models::Job;

fn config() -> Result<Config, Box<dyn std::error::Error>> {
    config::load_config("./config.toml")
}

pub async fn list_jobs() -> Result<Vec<Job>, Box<dyn std::error::Error>> {
    let mut configuration = Configuration::new();
    configuration.bearer_access_token = Some(config()?.hcloud_api_token);

    let servers = servers_api::list_servers(&configuration, Default::default())
        .await?
        .servers;

    let jobs = servers
        .into_iter()
        .map(|server| Job {
            id: server.id,
            ipv4: server.public_net.ipv4.unwrap().ip,
        })
        .collect();

    Ok(jobs)
}

pub async fn get_job(id: i64) -> Result<Option<Job>, Box<dyn std::error::Error>> {
    let mut configuration = Configuration::new();
    configuration.bearer_access_token = Some(config()?.hcloud_api_token);

    let server = servers_api::get_server(
        &configuration,
        hcloud::apis::servers_api::GetServerParams { id },
    )
    .await?
    .server;

    match server {
        Some(server) => Ok(Some(Job {
            id: server.id,
            ipv4: server.public_net.ipv4.unwrap().ip,
        })),
        None => Ok(None),
    }
}

pub async fn start_job(name: String) -> Result<Job, Box<dyn std::error::Error>> {
    let config = config()?;
    let mut configuration = Configuration::new();
    configuration.bearer_access_token = Some(config.hcloud_api_token);

    let params = CreateServerParams {
        create_server_request: Some(CreateServerRequest {
            name,
            image: config.image,
            server_type: config.server_type,
            location: Some(config.location),
            ssh_keys: Some(vec![config.ssh_key_name]),
            ..Default::default()
        }),
    };
    let res = servers_api::create_server(&configuration, params).await?;

    let job = Job {
        id: res.server.id,
        ipv4: res.server.public_net.ipv4.unwrap().ip,
    };

    Ok(job)
}

pub async fn stop_job(id: i64) -> Result<(), Box<dyn Error>> {
    let config = config()?;
    let mut configuration = Configuration::new();
    configuration.bearer_access_token = Some(config.hcloud_api_token);
    let params = DeleteServerParams { id };
    servers_api::delete_server(&configuration, params).await?;

    Ok(())
}

pub async fn tail(id: i64, filename: String) -> Result<(), Box<dyn Error>> {
    let config = config()?;
    let mut configuration = Configuration::new();
    configuration.bearer_access_token = Some(config.hcloud_api_token);

    let job = get_job(id).await?;

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
