use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use macros::generate_structs_from_ddl;
use serde::Deserialize;
use serde::Serialize;
use sqlx::postgres::PgPool;
use std::error::Error;
use std::future::Future;
#[derive(Serialize, Debug)]
struct User {
    id: u32,
    name: String,
    email: String,
}

generate_structs_from_ddl!("../openapi/ddl.sql");
async fn get_users() -> impl Responder {
    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        },
        User {
            id: 3,
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
        },
    ];
    let mut response = HttpResponse::Ok();
    response.set_header("Content-Type", "application/json");
    println!("{:?}", users[0]);
    response.json(users)
}

pub async fn create_pool() -> PgPool {
    let database_url = "postgres://postgres:password@localhost:5432/public";
    PgPool::connect(database_url)
        .await
        .expect("Failed to create pool")
}
#[get("/TEST")]
async fn get_test(pool: web::Data<PgPool>) -> impl Responder {
    let res: Vec<TEST> = getTEST(pool.get_ref()).await.unwrap();
    let mut response = HttpResponse::Ok();
    response.insert_header(("Content-Type", "application/json"));
    response.json(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = create_pool().await;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(get_test)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
