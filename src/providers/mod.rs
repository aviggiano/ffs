use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;

use async_trait::async_trait;
use ssh2::Session;

use crate::jobs::Job;

const INSTALL_SCRIPT: &str = include_str!(".././setup.sh");

fn install_over_ssh(
    ip: &str,
    key_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Attempting to connect to {ip} for setup installation...");

    // Retry SSH connection up to 30 times with 10 second delays (5 minutes total)
    let mut last_error = None;
    for attempt in 1..=30 {
        println!("SSH connection attempt {attempt}/30 to {ip}...");

        match try_install_ssh(ip, key_path) {
            Ok(()) => {
                println!("Successfully installed setup.sh on {ip}");
                return Ok(());
            }
            Err(e) => {
                println!("Attempt {attempt} failed: {e}");
                last_error = Some(e);
                if attempt < 30 {
                    std::thread::sleep(std::time::Duration::from_secs(10));
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "Failed to connect after 30 attempts".into()))
}

fn try_install_ssh(
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

    let exit_status = channel.exit_status()?;
    if exit_status != 0 {
        return Err(format!("Setup script exited with status {exit_status}").into());
    }

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

pub struct ProviderFactory;

impl ProviderFactory {
    /// Creates a new provider instance based on the provider name.
    ///
    /// # Panics
    ///
    /// Panics if an invalid provider name is provided.
    #[must_use]
    pub fn create_provider(name: &str) -> Box<dyn Provider> {
        match name {
            "hetzner" => Box::new(hetzner::HetznerProvider::new()),
            "aws" => Box::new(aws::AWSProvider::new()),
            _ => panic!("Invalid provider"),
        }
    }
}

pub mod aws;
pub mod hetzner;
