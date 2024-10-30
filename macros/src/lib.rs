extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use syn::{parse_macro_input, ItemImpl};

#[derive(Deserialize)]
struct FunctionDefinition {
    name: String,
    body: String,
}

#[proc_macro_attribute]
pub fn add_functions_from_file(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute input for the file path
    let relative_path = parse_macro_input!(attr as syn::LitStr).value();

    // Construct the absolute path
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let file_path = PathBuf::from(manifest_dir).join(relative_path);

    // Read the JSON file
    let file_content = fs::read_to_string(file_path).expect("Unable to read file");
    let functions: Vec<FunctionDefinition> =
        serde_json::from_str(&file_content).expect("JSON was not well-formatted");

    let input = parse_macro_input!(item as ItemImpl);
    let mut output = quote! { #input };

    // Generate functions based on the JSON data
    for func in functions {
        let func_name = syn::Ident::new(&func.name, proc_macro2::Span::call_site());
        let func_body = &func.body;

        let new_function = quote! {
            impl MyStruct {
                fn #func_name() {
                    println!("{}", #func_body);
                }
            }
        };

        output.extend(new_function);
    }

    TokenStream::from(output)
}
