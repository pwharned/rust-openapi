use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use macros::generate_structs_from_ddl;
use serde::Deserialize;
use serde::Serialize;
use sqlx::postgres::PgPool;
use std::error::Error;
use std::future::Future;

generate_structs_from_ddl!("../openapi/ddl.sql");

pub async fn create_pool() -> PgPool {
    let database_url = "postgres://postgres:password@localhost:5432/public";
    PgPool::connect(database_url)
        .await
        .expect("Failed to create pool")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = create_pool().await;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(get_test_handler)
            .service(post_test_handler)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
