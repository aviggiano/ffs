use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;

use async_trait::async_trait;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ec2::client::Waiters;
use aws_sdk_ec2::config::Region;
use aws_sdk_ec2::types::{InstanceType, ResourceType, Tag, TagSpecification};
use aws_sdk_ec2::Client;
use ssh2::Session;

use super::super::config;
use super::super::config::Config;
use super::Provider;
use crate::jobs::Job;

#[derive(Clone)]
pub struct AWSProvider {}

impl AWSProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Provider for AWSProvider {
    async fn start_job(&self, name: &str) -> Result<Job, Box<dyn std::error::Error + Send + Sync>> {
        let cfg = config()?;

        let region_provider =
            RegionProviderChain::first_try(Some(Region::new(cfg.location.clone())))
                .or_default_provider();
        let shared_config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&shared_config);

        let tag_spec = TagSpecification::builder()
            .resource_type(ResourceType::Instance)
            .tags(Tag::builder().key("Name").value(name).build())
            .build();

        let run_out = client
            .run_instances()
            .image_id(cfg.image.clone())
            .instance_type(InstanceType::from(cfg.server_type.as_str()))
            .min_count(1)
            .max_count(1)
            .key_name(cfg.ssh_key_name.clone())
            .tag_specifications(tag_spec)
            .send()
            .await?;

        let instance = run_out.instances().first().ok_or("no instance created")?;

        let instance_id = instance
            .instance_id()
            .ok_or("missing instance id")?
            .to_string();

        // Wait until instance is running
        let _ = client
            .wait_until_instance_running()
            .instance_ids(instance_id.clone())
            .wait(std::time::Duration::from_secs(120))
            .await;

        let desc = client
            .describe_instances()
            .instance_ids(instance_id.clone())
            .send()
            .await?;

        let ipv4 = desc
            .reservations()
            .first()
            .and_then(|res| res.instances().first())
            .and_then(|inst| inst.public_ip_address())
            .unwrap_or("")
            .to_string();

        let job = Job {
            id: instance_id.clone(),
            ipv4: ipv4.clone(),
            name: Some(name.to_string()),
        };

        let key_path = cfg.ssh_key_path.clone();
        tokio::spawn(async move {
            let _ = tokio::task::spawn_blocking(move || super::install_over_ssh(&ipv4, &key_path))
                .await;
        });

        Ok(job)
    }

    async fn get_job(
        &self,
        job_id: &str,
    ) -> Result<Option<Job>, Box<dyn std::error::Error + Send + Sync>> {
        let cfg = config()?;
        let region_provider =
            RegionProviderChain::first_try(Some(Region::new(cfg.location.clone())))
                .or_default_provider();
        let shared_config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&shared_config);

        let desc = client
            .describe_instances()
            .instance_ids(job_id)
            .send()
            .await?;

        if let Some(reservation) = desc.reservations().first() {
            if let Some(instance) = reservation.instances().first() {
                let name_tag = instance
                    .tags()
                    .iter()
                    .find(|t| t.key() == Some("Name"))
                    .and_then(|t| t.value())
                    .map(ToString::to_string);

                return Ok(Some(Job {
                    id: instance.instance_id().unwrap_or_default().to_string(),
                    ipv4: instance.public_ip_address().unwrap_or_default().to_string(),
                    name: name_tag,
                }));
            }
        }

        Ok(None)
    }

    async fn stop_job(
        &self,
        job_id: &str,
    ) -> Result<Job, Box<dyn std::error::Error + Send + Sync>> {
        let cfg = config()?;
        let region_provider =
            RegionProviderChain::first_try(Some(Region::new(cfg.location.clone())))
                .or_default_provider();
        let shared_config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&shared_config);

        client
            .terminate_instances()
            .instance_ids(job_id)
            .send()
            .await?;

        Ok(Job {
            id: job_id.to_string(),
            ipv4: String::new(),
            name: None,
        })
    }

    async fn list_jobs(&self) -> Result<Vec<Job>, Box<dyn std::error::Error + Send + Sync>> {
        let cfg = config()?;
        let region_provider =
            RegionProviderChain::first_try(Some(Region::new(cfg.location.clone())))
                .or_default_provider();
        let shared_config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&shared_config);

        let desc = client.describe_instances().send().await?;

        let mut jobs = Vec::new();
        for reservation in desc.reservations() {
            for instance in reservation.instances() {
                let name_tag = instance
                    .tags()
                    .iter()
                    .find(|t| t.key() == Some("Name"))
                    .and_then(|t| t.value())
                    .map(ToString::to_string);

                jobs.push(Job {
                    id: instance.instance_id().unwrap_or_default().to_string(),
                    ipv4: instance.public_ip_address().unwrap_or_default().to_string(),
                    name: name_tag,
                });
            }
        }

        Ok(jobs)
    }

    async fn tail(
        &self,
        job_id: &str,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cfg = config()?;

        if let Some(job) = self.get_job(job_id).await? {
            let tcp = TcpStream::connect((job.ipv4.as_str(), 22))?;
            let mut sess = Session::new()?;
            sess.set_tcp_stream(tcp);
            sess.handshake()?;
            sess.userauth_pubkey_file("root", None, Path::new(&cfg.ssh_key_path), None)?;

            let mut channel = sess.channel_session()?;
            channel.exec(&format!("cat {}", filename))?;

            let mut s = String::new();
            channel.read_to_string(&mut s)?;
            println!("{s}");
            channel.wait_close()?;
            println!("{}", channel.exit_status()?);
        }

        Ok(())
    }

    async fn scp(
        &self,
        _job_id: &str,
        _filename: &str,
        _destination: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

fn config() -> Result<Config, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Config::new())
}
