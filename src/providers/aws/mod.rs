use async_trait::async_trait;

use crate::jobs::Job;
use crate::providers::Provider;

pub struct AWSProvider {}

impl AWSProvider {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Provider for AWSProvider {
    async fn start_job(
        &self,
        job_id: &str,
    ) -> Result<Job, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Job {
            id: job_id.to_string(),
            ipv4: "".to_string(),
            name: None,
        })
    }

    async fn get_job(
        &self,
        _job_id: &str,
    ) -> Result<Option<Job>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    async fn stop_job(
        &self,
        job_id: &str,
    ) -> Result<Job, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Job {
            id: job_id.to_string(),
            ipv4: "".to_string(),
            name: None,
        })
    }

    async fn list_jobs(&self) -> Result<Vec<Job>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![])
    }

    async fn tail(
        &self,
        _job_id: &str,
        _filename: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}
