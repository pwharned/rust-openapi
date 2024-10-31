extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use syn::{parse_macro_input, ItemImpl};
#[proc_macro_attribute]
pub fn add_functions_from_file(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute input for the file path
    let relative_path = parse_macro_input!(attr as syn::LitStr).value();

    // Construct the absolute path
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let file_path = PathBuf::from(manifest_dir).join(relative_path);

    // Read the JSON file
    let file_content = fs::read_to_string(file_path).expect("Unable to read file");

    let openapi: OpenApiSpec =
        serde_json::from_str(&file_content).expect("Json was not well formatted");

    let input = parse_macro_input!(item as ItemImpl);
    let mut output = quote! { #input };

    /*
    * async fn get_data() -> Result<Value, reqwest::Error> {
        let client = Client::new();
        let response = client.get("https://api.example.com/data")
            .send()
            .await?;

        let data = response.json::<Value>().await?;
        Ok(data)
    }*/
    // Generate functions based on the JSON data
    for path_item in openapi.paths.paths {
        println!("{}", path_item.0);

        for item in path_item.1.methods {
            let method_name: &str = item.0.as_ref();
            let func_name: String = format!(
                "{}{}",
                method_name,
                path_item
                    .0
                    .replace('/', "_")
                    .replace('{', "by_")
                    .replace('}', ""),
            );
            let (parts_not_in_brackets, parts_in_brackets) = extract_parts(&path_item.0);
            let arg_names: Vec<syn::Ident> = parts_in_brackets
                .iter()
                .map(|arg| syn::Ident::new(arg, proc_macro2::Span::call_site()))
                .collect();

            let arg_types: Vec<syn::Type> = parts_in_brackets
                .iter()
                .map(|_| syn::parse_str::<syn::Type>("&String").unwrap())
                .collect();

            let args_iter = arg_names.iter().zip(arg_types.iter());
            let func_args: Vec<proc_macro2::TokenStream> = args_iter
                .map(|(name, ty)| {
                    quote! { #name: #ty }
                })
                .collect();

            let impl_name = syn::Ident::new(&func_name, proc_macro2::Span::call_site());
            let meth_name = syn::Ident::new(method_name, proc_macro2::Span::call_site());
            let new_function = quote! {
                            impl ApiClient {

            async fn #impl_name (&self, #(#func_args),*) -> Result<Value, reqwest::Error> {

                            let func_name = stringify!(#impl_name);
                            let method_name =stringify!(#meth_name);


                        let method: Method = Method::from_bytes(method_name.as_bytes() ).unwrap();
                        let client = Client::new();


                        let req = client.request(method, self.get_host());
                    let response = req
                        .send()
                        .await?;

                    let data = response.json::<Value>().await?;
                    Ok(data)
                }

                            }
                        };

            output.extend(new_function);
        }
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

    TokenStream::from(output)
}

fn extract_parts(path: &str) -> (String, Vec<String>) {
    let re = Regex::new(r"\{([^}]+)\}").unwrap();
    let mut parts_in_brackets = Vec::new();
    let mut parts_not_in_brackets = String::new();

    let mut last_end = 0;

    for cap in re.captures_iter(path) {
        let start = cap.get(0).unwrap().start();
        let end = cap.get(0).unwrap().end();

        // Append the part not in brackets
        parts_not_in_brackets.push_str(&path[last_end..start]);

        // Capture the part in brackets
        parts_in_brackets.push(cap.get(1).unwrap().as_str().to_string());

        last_end = end;
    }

    // Append the remaining part not in brackets
    parts_not_in_brackets.push_str(&path[last_end..]);

    (parts_not_in_brackets, parts_in_brackets)
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
    #[serde(flatten)]
    methods: std::collections::HashMap<String, Option<Operation>>,
    //post: Option<Operation>,
    //put: Option<Operation>,
    //delete: Option<Operation>,
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
