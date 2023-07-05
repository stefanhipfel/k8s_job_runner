// Copyright 2020-2022 The NATS Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
mod actions;
mod database_queue;
mod error;
mod handlers;
mod models;
mod queue;
mod schema;
mod worker;

use actix_web::{web, App, HttpServer};
use database_queue::DatabaseQueue;
use diesel::{
    r2d2::{self, ConnectionManager},
    SqliteConnection,
};
use queue::Queue;
use std::sync::Arc;

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=debug");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // create db connection pool
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let queue = Arc::new(DatabaseQueue::new(pool.clone()));
    let worker_queue = queue.clone(); // queue is an Arc pointer, so we only copy the reference

    tokio::spawn(async move { worker::run_worker(worker_queue).await });

    HttpServer::new(move || {
        let q: Arc<dyn Queue> = queue.clone();
        let store_queue: web::Data<dyn Queue> = web::Data::from(q);
        App::new()
            .app_data(store_queue)
            .service(web::scope("/internal").service(handlers::create_maintenance))
            .service(
                web::scope("/external")
                    .service(handlers::get_all_maintenance)
                    .service(handlers::get_maintenance),
            )
    })
    .shutdown_timeout(30)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
