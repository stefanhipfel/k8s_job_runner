use crate::error;
use crate::models::{Job, Maintenance};
use crate::queue::Queue;
use futures::{stream, StreamExt};
use k8s_openapi::api::batch::v1::Job as k8s_job;
use kube::{
    api::{Api, DeleteParams, PostParams},
    runtime::wait::{await_condition, Condition},
    Client,
};
use log::{debug, error, info};
use std::{sync::Arc, time::Duration};
use tokio;

const CONCURRENCY: usize = 50;

pub async fn run_worker(queue: Arc<dyn Queue>) {
    loop {
        let jobs = match queue.pull(CONCURRENCY as u32).await {
            Ok(jobs) => jobs,
            Err(err) => {
                error!("run_worker: pulling jobs: {}", err);
                tokio::time::sleep(Duration::from_secs(10)).await;
                Vec::new()
            }
        };

        let number_of_jobs = jobs.len();
        if number_of_jobs > 0 {
            debug!("Fetched {} jobs", number_of_jobs);
        }
        let cloned_queue = queue.clone();
        tokio::spawn(async move {
            stream::iter(jobs)
                .for_each_concurrent(CONCURRENCY, |job| async {
                    let job_id = job.0.uuid.to_string();
                    let res = match handle_job(job.0.clone(), job.1).await {
                        Ok(_) => cloned_queue.finish_job(job_id).await,
                        Err(err) => {
                            info!("run_worker: handling job({}): {}", job_id, &err);
                            cloned_queue.fail_job(job_id).await
                        }
                    };
                    match res {
                        Ok(_) => {
                            if let Some(err) = cleanup_job(job.0).await.err() {
                                error!("error deleting k8s job {}", err)
                            }
                        }
                        Err(err) => {
                            error!("run_worker: deleting / failing job: {}", err);
                        }
                    }
                })
                .await;
        });
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

async fn handle_job(job: Maintenance, job_type: Job) -> Result<(), crate::error::Error> {
    println!("{:?} JOB Started", job.uuid);
    let client = Client::try_default().await?;
    let jobs: Api<k8s_job> = Api::default_namespaced(client);

    info!("creating k8s job: {:?}", job.uuid);
    let name = format!("lifecycle_mgmt_{}", job.uuid);
    let data = match serde_json::from_value(serde_json::json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": name,
        },
        "spec": {
            "template": {
                "metadata": {
                    "name": name
                },
                "spec": {
                    "containers": [{
                        "name": "empty",
                        "image": job_type.docker_image
                    }],
                    "restartPolicy": "Never",
                }
            }
        }
    })) {
        Ok(j) => j,
        Err(e) => {
            error!("error creating k8s job: {}", e);
            return Err(error::Error::Internal(e.to_string()));
        }
    };
    jobs.create(&PostParams::default(), &data).await?;

    info!("Waiting for job to complete");
    let cond = await_condition(jobs.clone(), &name, is_job_completed_or_failed());

    let result = match tokio::time::timeout(std::time::Duration::from_secs(20), cond).await {
        Ok(j) => j,
        Err(e) => {
            jobs.delete(&name, &DeleteParams::background()).await?;
            return Err(error::Error::Internal(e.to_string()));
        }
    }?;

    if let Some(k8s_job) = result {
        if let Some(status) = k8s_job.status {
            if let Some(failed) = status.failed {
                if failed > 0 {
                    jobs.delete(&name, &DeleteParams::background()).await?;
                    return Err(error::Error::Internal("job failed".to_string()));
                }
            }
        }
    }
    info!("{:?} JOB finished", job.uuid);
    Ok(())
}

async fn cleanup_job(job: Maintenance) -> Result<(), crate::error::Error> {
    let client = Client::try_default().await?;
    let jobs: Api<k8s_job> = Api::default_namespaced(client);

    jobs.delete(&job.uuid, &DeleteParams::default())
        .await?
        .map_left(|o| debug!("Deleting job: {:?}", o.status))
        .map_right(|s| debug!("Deleted job: {:?}", s));

    Ok(())
}

#[must_use]
fn is_job_completed_or_failed() -> impl Condition<k8s_job> {
    |obj: Option<&k8s_job>| {
        if let Some(job) = &obj {
            if let Some(s) = &job.status {
                if let Some(conds) = &s.conditions {
                    if let Some(pcond) = conds.iter().find(|c| c.type_ == "Complete") {
                        return pcond.status == "True";
                    }
                    if let Some(pcond) = conds.iter().find(|c| c.type_ == "Failed") {
                        return pcond.status == "True";
                    }
                }
            }
        }
        false
    }
}
