extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};
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

#[proc_macro]
pub fn generate_structs_from_file(attr: TokenStream) -> TokenStream {
    // Parse the attribute input for the file path
    let relative_path = parse_macro_input!(attr as syn::LitStr).value();

    // Construct the absolute path
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let file_path = PathBuf::from(manifest_dir).join(relative_path);

    // Read the JSON file
    let file_content = fs::read_to_string(file_path).expect("Unable to read file");

    let openapi: OpenApiSpec =
        serde_json::from_str(&file_content).expect("Json was not well formatted");

    // Generate structs based on the JSON data
    let mut output = quote! {};

    for struct_def in openapi.components.unwrap().schemas.unwrap() {
        let struct_name = syn::Ident::new(&struct_def.0, proc_macro2::Span::call_site());
        let properties = struct_def.1.properties.unwrap();

        let fields = properties.iter().map(|field| {
            let field_name = syn::Ident::new(field.0, proc_macro2::Span::call_site());

            let field_type_str = field.1.type_.as_ref().unwrap().to_string();
            let field_ty: syn::Type = match field_type_str.as_str() {
                "string" => syn::parse_str("String").expect("Invalid type"),
                "integer" => syn::parse_str("i32").expect("Invalid type"),
                "boolean" => syn::parse_str("bool").expect("Invalid type"),
                // Handle other cases as needed
                _ => panic!("Unsupported type"),
            };

            quote! {
                pub #field_name: #field_ty,
            }
        });

        let new_struct = quote! {
            pub struct #struct_name {
                #(#fields)*
            }
        };

        output.extend(new_struct);
    }

    // Print the generated code }

    // Print the generated code for debugging
    println!("{}", output);

    TokenStream::from(output)
}

#[derive(Serialize, Deserialize, Debug)]
struct OpenApiSpec {
    openapi: String,
    info: Info,
    servers: Option<Vec<Server>>,
    paths: Paths,
    components: Option<Components>,
    // Add other fields as needed
}

#[derive(Serialize, Deserialize, Debug)]
struct Info {
    title: String,
    description: Option<String>,
    version: String,
    // Add other fields as needed
}

#[derive(Serialize, Deserialize, Debug)]
struct Server {
    url: String,
    description: Option<String>,
    // Add other fields as needed
}

#[derive(Serialize, Deserialize, Debug)]
struct Paths {
    #[serde(flatten)]
    paths: std::collections::HashMap<String, PathItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PathItem {
    get: Option<Operation>,
    post: Option<Operation>,
    put: Option<Operation>,
    delete: Option<Operation>,
    // Add other HTTP methods as needed
}

#[derive(Serialize, Deserialize, Debug)]
struct Operation {
    tags: Option<Vec<String>>,
    summary: Option<String>,
    description: Option<String>,
    operation_id: Option<String>,
    parameters: Option<Vec<Parameter>>,
    request_body: Option<RequestBody>,
    responses: Responses,
    // Add other fields as needed
}

#[derive(Serialize, Deserialize, Debug)]
struct Parameter {
    name: String,
    #[serde(rename = "in")]
    in_: String,
    description: Option<String>,
    required: Option<bool>,
    schema: Option<Schema>,
    // Add other fields as needed
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestBody {
    content: std::collections::HashMap<String, MediaType>,
    // Add other fields as needed
}

#[derive(Serialize, Deserialize, Debug)]
struct MediaType {
    schema: Schema,
    // Add other fields as needed
}

#[derive(Serialize, Deserialize, Debug)]
struct Responses {
    #[serde(flatten)]
    responses: std::collections::HashMap<String, Response>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    description: String,
    content: Option<std::collections::HashMap<String, MediaType>>,
    // Add other fields as needed
}

#[derive(Serialize, Deserialize, Debug)]
struct Components {
    schemas: Option<std::collections::HashMap<String, Schema>>,
    // Add other fields as needed
}

#[derive(Serialize, Deserialize, Debug)]
struct Schema {
    #[serde(rename = "type")]
    type_: Option<String>,
    properties: Option<std::collections::HashMap<String, Property>>,
    // Add other fields as needed
}
#[derive(Serialize, Deserialize, Debug)]
struct Property {
    #[serde(rename = "type")]
    type_: Option<String>,
    format: Option<String>,
    // Add other fields as needed
}
