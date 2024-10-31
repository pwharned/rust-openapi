use macros::add_functions_from_file;
use macros::generate_structs_from_file;
use reqwest::Client;
use reqwest::Error;
use serde_json::Value;
generate_structs_from_file!("openapi.json");

struct MyStruct;
#[add_functions_from_file("openapi.json")]
impl MyStruct {
    fn existing_function(&self) {
        println!("This is an existing function.");
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

    let my_struct = MyStruct;
    my_struct.existing_function();
    MyStruct::get_users_by_id().await?; // Calling the new function added by the macro
    Ok(())
}
