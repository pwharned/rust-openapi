use macros::add_functions_from_file;

use macros::generate_structs_from_file;
generate_structs_from_file!("openapi.json");

struct MyStruct;
#[add_functions_from_file("funcs.json")]
impl MyStruct {
    fn existing_function(&self) {
        println!("This is an existing function.");
    }
}

fn main() {
    let person = User {
        email: String::from("Alice"),
        name: String::from("test"),
        id: 30,
    };
    println!("Name: {}, Age: {}", person.name, person.id);

    let my_struct = MyStruct;
    my_struct.existing_function();
    MyStruct::func3(); // Calling the new function added by the macro
}
