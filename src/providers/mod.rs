use crate::jobs::Job;

pub enum ProviderType {
    Hetzner,
    AWS,
}

pub trait Provider {
    async fn start_job(&self, name: &str) -> Result<Job, Box<dyn std::error::Error>>;
    async fn get_job(&self, job_id: &str) -> Result<Option<Job>, Box<dyn std::error::Error>>;
    async fn stop_job(&self, job_id: &str) -> Result<Job, Box<dyn std::error::Error>>;
    async fn list_jobs(&self) -> Result<Vec<Job>, Box<dyn std::error::Error>>;
    async fn tail(&self, job_id: &str, filename: &str) -> Result<(), Box<dyn std::error::Error>>;
}

pub mod aws;
pub mod hetzner;
