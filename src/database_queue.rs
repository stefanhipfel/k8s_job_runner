use crate::actions;
use crate::models::{Job, JobStatus, Maintenance};
use crate::queue::Queue;
use crate::DbPool;

#[derive(Debug, Clone)]
pub struct DatabaseQueue {
    db: DbPool,
    max_attempts: u32,
}

impl DatabaseQueue {
    pub fn new(db: DbPool) -> DatabaseQueue {
        let queue = DatabaseQueue {
            db,
            max_attempts: 5,
        };

        queue
    }
}

#[async_trait::async_trait]
impl Queue for DatabaseQueue {
    fn push(&self, mut job: Maintenance) -> Result<(), crate::error::Error> {
        //let scheduled_for = date.unwrap_or(chrono::Utc::now());
        job.failed_attempts = 0;
        job.status = JobStatus::Queued.to_string();
        let mut conn = self.db.get().unwrap();
        actions::insert_new_maintenance(&mut conn, job)?;
        Ok(())
    }

    async fn delete_job(&self, job_id: String) -> Result<(), crate::error::Error> {
        let mut conn = self.db.get().unwrap();
        actions::delete_maintenance(&mut conn, job_id)?;
        Ok(())
    }
    //
    async fn fail_job(&self, job_id: String) -> Result<(), crate::error::Error> {
        let mut conn = self.db.get().unwrap();
        actions::update_maintenance_status(&mut conn, job_id, JobStatus::Failed)?;
        Ok(())
    }

    async fn finish_job(&self, job_id: String) -> Result<(), crate::error::Error> {
        let mut conn = self.db.get().unwrap();
        actions::update_maintenance_status(&mut conn, job_id, JobStatus::Finished)?;
        Ok(())
    }

    async fn pull(
        &self,
        number_of_jobs: u32,
    ) -> Result<Vec<(Maintenance, Job)>, crate::error::Error> {
        let number_of_jobs = if number_of_jobs > 100 {
            100
        } else {
            number_of_jobs
        };
        let mut conn = self.db.get().unwrap();
        let jobs = actions::get_ready_maintenance_jobs(&mut conn)?;

        if let Some(jobs) = jobs {
            Ok(jobs)
        } else {
            Ok(Vec::new())
        }
    }

    async fn clear(&self) -> Result<(), crate::error::Error> {
        let mut conn = self.db.get().unwrap();
        actions::delete_all_maintenance(&mut conn)?;
        Ok(())
    }
}
