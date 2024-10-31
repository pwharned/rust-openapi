use macros::add_functions_from_file;
use macros::generate_structs_from_file;
use reqwest::Error;
use reqwest::{Client, Method};
use serde_json::Value;
generate_structs_from_file!("openapi.json");

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
        host: "http://localhost".to_string(),
    };
    apiclient.get_users_by_id(&"hello".to_string()).await?; // Calling the new function added by the macro
    Ok(())
}
