use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;

#[derive(Serialize, Debug)]
struct User {
    id: u32,
    name: String,
    email: String,
}

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/users", web::get().to(get_users)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
