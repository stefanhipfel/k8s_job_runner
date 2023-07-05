use crate::models::Job;
use crate::models::Maintenance;
use async_trait;
use std::fmt::Debug;

#[async_trait::async_trait]
pub trait Queue: Send + Sync + Debug {
    fn push(&self, job: Maintenance) -> Result<(), crate::error::Error>;
    /// pull fetches at most `number_of_jobs` from the queue.
    async fn pull(
        &self,
        number_of_jobs: u32,
    ) -> Result<Vec<(Maintenance, Job)>, crate::error::Error>;
    async fn delete_job(&self, job_id: String) -> Result<(), crate::error::Error>;
    async fn fail_job(&self, job_id: String) -> Result<(), crate::error::Error>;
    async fn finish_job(&self, job_id: String) -> Result<(), crate::error::Error>;
    async fn clear(&self) -> Result<(), crate::error::Error>;
}
