use crate::models;
use diesel::dsl::{delete, insert_into, now, update};
use diesel::prelude::*;
use diesel::result::Error as dieselError;

pub fn get_all_maintenance(
    conn: &mut SqliteConnection,
) -> Result<Option<Vec<models::Maintenance>>, dieselError> {
    use crate::schema::maintenances::dsl::*;

    let maint = maintenances.load::<models::Maintenance>(conn).optional()?;
    Ok(maint)
}

pub fn find_maintenance_by_os_uuid(
    conn: &mut SqliteConnection,
    uid: String,
) -> Result<Option<models::Maintenance>, dieselError> {
    use crate::schema::maintenances::dsl::*;

    let maint = maintenances
        .filter(uuid.eq(uid.to_string()))
        .first::<models::Maintenance>(conn)
        .optional()?;

    Ok(maint)
}

pub fn insert_new_maintenance(
    conn: &mut SqliteConnection,
    object: models::Maintenance,
) -> Result<models::Maintenance, dieselError> {
    use crate::schema::maintenances::dsl::*;
    insert_into(maintenances)
        .values(&object)
        .on_conflict(uuid)
        .do_update()
        .set(&object)
        .get_result(conn)
}

pub fn insert_new_job(
    conn: &mut SqliteConnection,
    object: models::Job,
) -> Result<models::Job, dieselError> {
    use crate::schema::jobs::dsl::*;
    insert_into(jobs)
        .values(&object)
        .on_conflict((name, docker_image, docker_image_tag))
        .do_update()
        .set(&object)
        .get_result(conn)
}

pub fn delete_maintenance(conn: &mut SqliteConnection, uid: String) -> Result<(), dieselError> {
    use crate::schema::maintenances::dsl::*;
    delete(maintenances)
        .filter(uuid.eq(uid.to_string()))
        .execute(conn)?;

    Ok(())
}

pub fn delete_all_maintenance(conn: &mut SqliteConnection) -> Result<(), dieselError> {
    use crate::schema::maintenances::dsl::*;
    delete(maintenances).execute(conn)?;

    Ok(())
}

pub fn update_maintenance_status(
    conn: &mut SqliteConnection,
    uid: String,
    job_status: models::JobStatus,
) -> Result<(), dieselError> {
    use crate::schema::maintenances::dsl::*;
    update(maintenances)
        .filter(uuid.eq(uid.to_string()))
        .set((
            status.eq(job_status.to_string()),
            updated_at.eq(now),
            failed_attempts.eq(failed_attempts + 1),
        ))
        .execute(conn)?;

    Ok(())
}

pub fn get_ready_maintenance_jobs(
    conn: &mut SqliteConnection,
) -> Result<Option<Vec<(models::Maintenance, models::Job)>>, dieselError> {
    use crate::schema::jobs::dsl::*;
    use crate::schema::maintenances::dsl::*;
    //AND failed_attempts < $5

    let page_with_book = jobs
        .inner_join(maintenances)
        .filter(status.eq(models::JobStatus::Queued.to_string()))
        .filter(scheduled_for.le(now))
        .select((models::Maintenance::as_select(), models::Job::as_select()))
        .get_results::<(models::Maintenance, models::Job)>(conn)
        .optional()?;

    update(maintenances)
        .filter(status.eq(models::JobStatus::Queued.to_string()))
        .filter(scheduled_for.le(now))
        .set((
            status.eq(models::JobStatus::Running.to_string()),
            updated_at.eq(now),
        ))
        .execute(conn)?;

    Ok(page_with_book)
}
