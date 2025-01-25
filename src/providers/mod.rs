use async_trait::async_trait;

use crate::jobs::Job;

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
}

pub mod aws;
pub mod hetzner;
