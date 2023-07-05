use std::{fmt, str::FromStr};

//use super::schema::maintenances;
use crate::schema::jobs;
use crate::schema::maintenances;
use chrono::NaiveDateTime;
use diesel::Associations;
use diesel::Identifiable;
use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(
    Identifiable,
    Queryable,
    AsChangeset,
    Associations,
    Insertable,
    PartialEq,
    Clone,
    Serialize,
    Deserialize,
    Selectable,
    Debug,
)]
#[diesel(table_name = maintenances)]
#[belongs_to(Job)]
#[diesel(primary_key(id))]
pub struct Maintenance {
    #[serde(skip_deserializing)]
    pub id: Option<i32>,
    #[serde(skip_deserializing)]
    pub uuid: String,
    #[serde(skip_deserializing)]
    pub name: Option<String>,
    #[serde(skip_deserializing)]
    pub created_at: NaiveDateTime,
    #[serde(skip_deserializing)]
    pub updated_at: Option<NaiveDateTime>,
    #[serde(skip_deserializing)]
    pub failed_attempts: i32,
    #[serde(skip_deserializing)]
    pub status: String,
    pub scheduled_for: Option<NaiveDateTime>,
    pub downtime_window_start: Option<NaiveDateTime>,
    pub downtime_window_end: Option<NaiveDateTime>,
    pub job_id: i32,
}

#[derive(
    Queryable,
    AsChangeset,
    PartialEq,
    Clone,
    Serialize,
    Deserialize,
    Debug,
    Identifiable,
    Insertable,
    Selectable,
)]
#[diesel(table_name = jobs)]
#[diesel(primary_key(id))]
pub struct Job {
    pub id: i32,
    pub name: String,
    pub version: Option<String>,
    pub docker_image: String,
    pub docker_image_tag: String,
}

pub enum JobStatus {
    NotQueued,
    Queued,
    Running,
    Failed,
    Finished,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JobStatus::NotQueued => write!(f, "not_queued"),
            JobStatus::Queued => write!(f, "queued"),
            JobStatus::Running => write!(f, "running"),
            JobStatus::Failed => write!(f, "failed"),
            JobStatus::Finished => write!(f, "finished"),
        }
    }
}

impl FromStr for JobStatus {
    type Err = ();

    fn from_str(input: &str) -> Result<JobStatus, Self::Err> {
        match input {
            "not_queued" => Ok(JobStatus::NotQueued),
            "queued" => Ok(JobStatus::Queued),
            "running" => Ok(JobStatus::Running),
            "failed" => Ok(JobStatus::Failed),
            "finished" => Ok(JobStatus::Finished),
            _ => Err(()),
        }
    }
}
