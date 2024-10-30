use macros::add_functions_from_file;
struct MyStruct;
#[add_functions_from_file("funcs.json")]
impl MyStruct {
    fn existing_function(&self) {
        println!("This is an existing function.");
    }
}

fn main() {
    let my_struct = MyStruct;
    my_struct.existing_function();
    MyStruct::func3(); // Calling the new function added by the macro
}
