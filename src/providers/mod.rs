use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;

use async_trait::async_trait;
use ssh2::Session;

use crate::jobs::Job;

const INSTALL_SCRIPT: &str = r#"#!/bin/bash
set -e
apt-get update
apt-get install -y curl git build-essential pkg-config libssl-dev
curl -fsSL https://deb.nodesource.com/setup_lts.x | bash -
apt-get install -y nodejs
curl https://sh.rustup.rs -sSf | sh -s -- -y
source "$HOME/.cargo/env"
curl -L https://foundry.paradigm.xyz | bash
"$HOME"/.foundry/bin/foundryup
"$HOME"/.cargo/bin/cargo install --locked echidna
"$HOME"/.cargo/bin/cargo install --locked --git https://github.com/crytic/medusa
"$HOME"/.cargo/bin/cargo install --locked --git https://github.com/crytic/halmos
"#;

fn install_over_ssh(
    ip: &str,
    key_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tcp = TcpStream::connect((ip, 22))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_pubkey_file("root", None, Path::new(key_path), None)?;

    let mut channel = sess.channel_session()?;
    channel.exec("bash -s")?;
    channel.write_all(INSTALL_SCRIPT.as_bytes())?;
    channel.send_eof()?;
    channel.wait_close()?;

    Ok(())
}

pub enum ProviderType {
    Hetzner,
    AWS,
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn start_job(&self, name: &str) -> Result<Job, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_job(
        &self,
        job_id: &str,
    ) -> Result<Option<Job>, Box<dyn std::error::Error + Send + Sync>>;
    async fn stop_job(&self, job_id: &str)
        -> Result<Job, Box<dyn std::error::Error + Send + Sync>>;
    async fn list_jobs(&self) -> Result<Vec<Job>, Box<dyn std::error::Error + Send + Sync>>;
    async fn tail(
        &self,
        job_id: &str,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn scp(
        &self,
        job_id: &str,
        filename: &str,
        destination: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn new(name: &str) -> Box<dyn Provider> {
        match name {
            "hetzner" => Box::new(hetzner::HetznerProvider::new()),
            "aws" => Box::new(aws::AWSProvider::new()),
            _ => panic!("Invalid provider"),
        }
    }

    async fn install_dependencies(
        &self,
        ip: &str,
        key_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ip = ip.to_string();
        let key = key_path.to_string();
        tokio::task::spawn_blocking(move || install_over_ssh(&ip, &key)).await??;
        Ok(())
    }
}

pub mod aws;
pub mod hetzner;
