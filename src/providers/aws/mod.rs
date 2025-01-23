use std::error::Error;

use super::super::models::Job;

pub fn list_jobs() -> Result<Vec<Job>, Box<dyn std::error::Error>> {
    Ok(vec![])
}

pub async fn stop_job(_id: i64) -> Result<(), Box<dyn Error>> {
    Ok(())
}
