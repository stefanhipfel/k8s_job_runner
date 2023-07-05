use super::actions;
use super::models;
use crate::queue::Queue;
use crate::DbPool;
use actix_web::{
    get,
    http::{header::ContentType, StatusCode},
    put, web, Error, HttpResponse, ResponseError, Result,
};
use chrono::{DateTime, Utc};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenstackObject {
    uuid: Option<String>,
    planned: Option<DateTime<Utc>>,
    task: bool,
}

#[derive(Debug, Display, Error)]
enum UserError {
    #[display(fmt = "An internal error occurred. Please try again later.")]
    InternalError,
}

impl ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            UserError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/*
#[get("/maintenance/{uuid}")]
async fn get_data(
    client: web::Data<jetstream::kv::Store>,
    pool: web::Data<Pool<ConnectionManager<SqliteConnection>>>,
    uuid: web::Path<String>,
) -> Result<HttpResponse, UserError> {
    let uuid = uuid.into_inner();

    let entry = client
        .entry(uuid.clone())
        .await
        .map_err(|_e| UserError::InternalError)?;

    if let Some(entry) = entry {
        let result = serde_json::from_slice::<OpenstackObject>(&entry.value)
            .map_err(|_e| UserError::InternalError)?;
        return Ok(HttpResponse::Ok().json(result));
    }
    Err(UserError::InternalError)
}

#[put("/maintenance/{uuid}")]
async fn put_maintenance(
    client: web::Data<jetstream::kv::Store>,
    mut object: web::Json<OpenstackObject>,
    uuid: web::Path<String>,
) -> Result<HttpResponse, UserError> {
    let uuid = uuid.into_inner();
    object.uuid = Some(uuid.clone());
    let bytes = serde_json::to_vec(&json!(object)).map_err(|_e| UserError::InternalError)?;
    client
        .put(uuid.to_string(), bytes.into())
        .await
        .map_err(|_e| UserError::InternalError)?;

    return Ok(HttpResponse::Created().json(uuid.to_string()));
}
*/
#[get("/")]
pub async fn get_all_maintenance(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    // use web::block to offload blocking Diesel code without blocking server thread
    let maint = web::block(move || {
        let mut conn = pool.get().unwrap();
        actions::get_all_maintenance(&mut conn)
    })
    .await?
    .map_err(|_e| UserError::InternalError)?;

    if let Some(maint) = maint {
        Ok(HttpResponse::Ok().json(maint))
    } else {
        let res = HttpResponse::NotFound().body(format!("No objects found"));
        Ok(res)
    }
}

#[get("/show/{uuid}")]
pub async fn get_maintenance(
    pool: web::Data<DbPool>,
    os_uuid: web::Path<String>,
) -> Result<HttpResponse, Error> {
    // use web::block to offload blocking Diesel code without blocking server thread
    let maint = web::block(move || {
        let mut conn = pool.get().unwrap();
        actions::find_maintenance_by_os_uuid(&mut conn, os_uuid.to_string())
    })
    .await?
    .map_err(|_e| UserError::InternalError)?;

    if let Some(maint) = maint {
        Ok(HttpResponse::Ok().json(maint))
    } else {
        let res = HttpResponse::NotFound().body(format!("No objects found"));
        Ok(res)
    }
}

#[put("/maintenance/{uuid}")]
pub async fn create_maintenance(
    queue: web::Data<dyn Queue>,
    os_uuid: web::Path<String>,
    mut object: web::Json<models::Maintenance>,
) -> Result<HttpResponse, Error> {
    object.uuid = os_uuid.into_inner();
    object.id = None;
    let _result = web::block(move || queue.push(object.0)).await?;

    Ok(HttpResponse::Ok().finish())
}

#[put("/job/{uuid}")]
pub async fn create_job(
    pool: web::Data<DbPool>,
    object: web::Json<models::Job>,
) -> Result<HttpResponse, Error> {
    let result = web::block(move || {
        let mut conn = pool.get().unwrap();
        actions::insert_new_job(&mut conn, object.0)
    })
    .await?
    .map_err(|_e| UserError::InternalError)?;

    Ok(HttpResponse::Ok().json(result))
}
