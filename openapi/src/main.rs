use macros::add_functions_from_file;
use macros::generate_structs_from_ddl;
use macros::generate_structs_from_file;
use reqwest::Error;
use reqwest::{Client, Method};
use serde::Deserialize;
generate_structs_from_file!("openapi.json");
//generate_structs_from_ddl!("ddl.sql");

struct ApiClient {
    host: String,
}
#[add_functions_from_file("openapi.json")]
impl ApiClient {
    fn get_host(&self) -> &str {
        &self.host // Return a reference to the value
    }
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let person = User {
        email: String::from("Alice"),
        name: String::from("test"),
        id: 30,
    };
    println!("Name: {}, Age: {}", person.name, person.id);

    let apiclient = ApiClient {
        host: "http://localhost:8080".to_string(),
    };

    match apiclient.get_users().await {
        Ok(data) => println!("{:?}", data),
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}
