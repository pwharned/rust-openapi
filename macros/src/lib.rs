extern crate proc_macro;
use sqlx::FromRow;
mod parse;
use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use syn::LitStr;
use syn::Type;
use syn::{parse_macro_input, ItemImpl};
fn remove_values_inside_brackets(input: &str) -> String {
    let re = Regex::new(r"\{[^}]+\}").unwrap();
    let result = re.replace_all(input, "{}");
    "{}".to_string() + &result
}

fn extract_parts_helper(route: &str) -> (Vec<String>, Vec<String>) {
    let mut parts_not_in_brackets = Vec::new();
    let mut parts_in_brackets = Vec::new();
    let mut current_part = String::new();
    let mut in_brackets = false;

    for c in route.chars() {
        if c == '{' {
            if !current_part.is_empty() {
                current_part.push('{');
                current_part.push('}');

                parts_not_in_brackets.push(current_part.clone());
                current_part.clear();
            }
            in_brackets = true;
        } else if c == '}' {
            parts_in_brackets.push(current_part.clone());
            current_part.clear();
            in_brackets = false;
        } else {
            current_part.push(c);
        }
    }

    if !current_part.is_empty() {
        parts_not_in_brackets.push(current_part);
    }

    (parts_not_in_brackets, parts_in_brackets)
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

    let openapi: OpenApiSpec =
        serde_json::from_str(&file_content).expect("Json was not well formatted");

    // Extract the name of the struct
    let input = parse_macro_input!(item as ItemImpl);
    let mut output = quote! { #input };
    let struct_name = match *input.self_ty {
        Type::Path(ref type_path) => {
            if let Some(segment) = type_path.path.segments.first() {
                segment.ident.clone()
            } else {
                panic!("Expected a path segment");
            }
        }
        _ => panic!("Expected a type path"),
    };
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

            let (parts_not_in_brackets1, parts_in_brackets1) = extract_parts_helper(&path_item.0);

            let impl_name = syn::Ident::new(&func_name, proc_macro2::Span::call_site());
            let meth_name = syn::Ident::new(method_name, proc_macro2::Span::call_site());

            let blank_url = syn::parse::<LitStr>(
                remove_values_inside_brackets(&path_item.0)
                    .to_token_stream()
                    .into(),
            )
            .unwrap()
            .value();
            let mut new_function = proc_macro2::TokenStream::new();
            if arg_names.is_empty() {
                new_function = quote! {
                                impl #struct_name {
                async fn #impl_name (&self, #(#func_args),*) -> Result<Vec<User>, reqwest::Error> {

                                let func_name = stringify!(#impl_name);
                                let method_name =stringify!(#meth_name);

                        let base_url = self.get_host();
                            let url = format!(#blank_url, self.get_host());
                            let method: Method = Method::from_bytes(method_name.as_bytes() ).unwrap();
                            let client = Client::new();



                        let response = match method_name {
                            "GET" => client.get(url).send().await?,
                            "PATCH" => client.patch(url).send().await?,
                            "POST" => client.post(url).send().await?,
                            "PUT" => client.put(url).send().await?,

                                _ => reqwest::get(url).await?
                        };

                        let data = response.json::<Vec<User>>().await?;
                        Ok(data)
                    }

                                }
                            };
            } else {
                new_function = quote! {
                                impl #struct_name {
                async fn #impl_name (&self, #(#func_args),*) -> Result<Vec<User>, reqwest::Error> {

                                let func_name = stringify!(#impl_name);
                                let method_name =stringify!(#meth_name);
                        let base_url = format!(#blank_url,self.get_host(), #(#arg_names),* );
                            //let test_url = format!("{}", #(#arg_names),* );


                            let method: Method = Method::from_bytes(method_name.as_bytes() ).unwrap();
                            let client = Client::new();



                            let req = client.request(method, self.get_host());
                        let response = req
                            .send()
                            .await?;

                        let data = response.json::<Vec<User>>().await?;
                        Ok(data)
                    }

                                }
                            };
            }

            output.extend(new_function);
        }
    }

    //println!("{}", output);
    TokenStream::from(output)
}
#[proc_macro]
pub fn generate_structs_from_ddl(attr: TokenStream) -> TokenStream {
    // Parse the attribute input for the file path
    let relative_path = parse_macro_input!(attr as syn::LitStr).value();

    // Construct the absolute path
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let file_path = PathBuf::from(manifest_dir).join(relative_path);

    // Read the JSON file
    let file_content = fs::read_to_string(file_path).expect("Unable to read file");

    let sql = file_content.split(";");
    // Generate structs based on the JSON data
    let mut output = quote! {};

    for statement in sql.into_iter() {
        let stmt = parse::create_table_parser().parse(statement);
        if stmt.is_none() {
            continue;
        }
        //let create_table_result = create_table_parser().parse("CREATE TABLE TEST(id int, id2 int)");
        let ddl = stmt.unwrap();
        let table_name = &ddl.0 .0;
        let struct_name = syn::Ident::new(&ddl.0 .0, proc_macro2::Span::call_site());
        let columns = ddl.0 .1;

        let fields = columns.iter().map(|(colname, data_type, _)| {
            let field_name = syn::Ident::new(colname, proc_macro2::Span::call_site());

            let field_type_str = data_type.to_string();
            let field_ty: syn::Type = match field_type_str.as_str() {
                "VARCHAR" => syn::parse_str("String").expect("Invalid type"),
                "INT" => syn::parse_str("i32").expect("Invalid type"),
                "BOOL" => syn::parse_str("bool").expect("Invalid type"),
                // Handle other cases as needed
                _ => syn::parse_str("String").expect("Invalid type"),
            };

            quote! {
                pub #field_name: Option<#field_ty>,
            }
        });

        let fields2 = columns.iter().map(|(colname, _, _)| {
            let field_name = syn::Ident::new(colname, proc_macro2::Span::call_site());

            quote! {
                   .bind(json.#field_name)
            }
        });
        let fields3 = columns.iter().map(|(colname, _, _)| {
            let field_name = syn::Ident::new(colname, proc_macro2::Span::call_site());

            quote! {
                if !self.#field_name.is_none(){
                    fields.push((#colname, &self.#field_name as &dyn std::fmt::Debug) );
                }
            }
        });

        let cols = columns
            .iter()
            .map(|(colname, _, _)| *colname)
            .collect::<Vec<_>>()
            .join(",");
        let primary_key: Vec<(&str, &str)> = columns
            .iter()
            .filter(|(_, _, options)| !options.is_empty())
            .map(move |(colname, _, pkey)| (*colname, pkey[0]))
            .collect();

        let new_struct = quote! {
            #[derive(Deserialize,Serialize,Debug,sqlx::FromRow)]
            pub struct #struct_name {
                #(#fields)*
            }

        };

        let new_struct2 = quote! {
                       impl #struct_name {


               pub fn non_null_fields(&self) -> Vec<(&str, &dyn std::fmt::Debug)>{

                           let mut fields = Vec::new();
                           #(#fields3)*
                           fields
                       }
        pub fn bind_fields(&self,  sqlx_query:&mut sqlx::query::Query<sqlx::Postgres, sqlx::postgres::PgArguments> ) -> (){

                          // #(#fields2)*
                       }




                       }

                   };

        for k in primary_key {
            let key = k.0;
            let inner_fields = columns
                .iter()
                .filter(|(colname, _, _)| *colname != key)
                .map(|(colname, _, _)| {
                    let field_name = syn::Ident::new(colname, proc_macro2::Span::call_site());

                    quote! {
                           .bind(json.#field_name.unwrap())
                    }
                });

            let select = "SELECT ".to_owned()
                + &cols.to_owned()
                + " FROM "
                + &table_name.to_owned()
                + " WHERE "
                + key
                + " = $1";
            let del =
                "DELETE FROM ".to_owned() + &table_name.to_owned() + " WHERE " + key + " = $1";

            let get = "get_".to_owned() + &table_name.to_lowercase() + "_by_" + key;
            let route = "/".to_owned() + table_name + "/" + "{" + key + "}";
            let get_handler_function_name = get + "_handler";

            let get_handler_function_name_syn =
                syn::Ident::new(&get_handler_function_name, proc_macro2::Span::call_site());

            let key_syn = syn::Ident::new(key, proc_macro2::Span::call_site());
            let get_handler = quote! {
            #[get(#route)]
            async fn #get_handler_function_name_syn(path: web::Path<#struct_name>, pool: web::Data<PgPool>) -> impl Responder {
                    println!("{}", #select);
                    let v = path.into_inner();
                let res: Vec<#struct_name> = sqlx::query_as::<_,#struct_name>(#select).bind(v.#key_syn).fetch_all( pool.get_ref()).await.unwrap();
                let mut response = HttpResponse::Ok();
                response.insert_header(("Content-Type", "application/json"));
                response.json(res)
            }
                };
            let delete = "delete_".to_owned() + &table_name.to_lowercase() + "_by_" + key;
            let route = "/".to_owned() + table_name + "/" + "{" + key + "}";
            let delete_handler_function_name = delete + "_handler";

            let delete_handler_function_name_syn = syn::Ident::new(
                &delete_handler_function_name,
                proc_macro2::Span::call_site(),
            );

            let delete_handler = quote! {
            #[delete(#route)]
            async fn #delete_handler_function_name_syn(path: web::Path<#struct_name>, pool: web::Data<PgPool>) -> impl Responder {
                    println!("{}", #del);
                    let v = path.into_inner();
                let res = sqlx::query(#del).bind(v.#key_syn).execute( pool.get_ref()).await.unwrap();
                let mut response = HttpResponse::Ok();
                //response.insert_header(("Content-Type", "application/json"));
                //response.json(res)
                response
            }
                };

            output.extend(get_handler);
            output.extend(delete_handler);

            let update = "update_".to_owned() + &table_name.to_lowercase();
            let route = "/".to_owned() + table_name + "/" + "{" + key + "}";
            let update_handler_function_name = update + "_handler";

            let update_handler_function_name_syn = syn::Ident::new(
                &update_handler_function_name,
                proc_macro2::Span::call_site(),
            );

            let update_handler = quote! {
                                                                            #[patch(#route)]
                                                                            async fn #update_handler_function_name_syn(path: web::Path<#struct_name>, json: web::Json<#struct_name>, pool: web::Data<PgPool>) -> impl Responder {
                                                                        let active_fields : Vec<(&str, &dyn std::fmt::Debug)>= json.non_null_fields();
                                                                        let fields_length  = active_fields.len();
                                                                let insert_sql = "UPDATE ".to_owned() + &#table_name.to_owned()
                                                                                + " set " + &active_fields.into_iter().enumerate().filter(|(index, (name, value)) | *name!=#key ). map(|(index,(name,value)) | format!(" {} = ${} ", &name.to_string(), index+2)).collect::<Vec<_>>().join(" ").to_owned()
                                                                                + " where "
                                                                                 + #key + &format!(" = ${} ", fields_length).to_string();
                        println!("{}",insert_sql);
                                            let v= path.into_inner();

                                    let mut sqlx_query: sqlx::query::Query<sqlx::Postgres, sqlx::postgres::PgArguments> = sqlx::query(&insert_sql).bind(&v.#key_syn) #(#inner_fields)*;
                                            let result = sqlx_query.execute( pool.get_ref()).await;
                    println!("{}", json.id2.unwrap());
            match result { Ok(res) => { println!("Query executed successfully: {:?}", res); } Err(e) => {  println!("Error executing query: {:?}", e); } }

                                                                                let mut response = HttpResponse::Ok();
                                                                                //response.insert_header(("Content-Type", "application/json"));
                                                                                //response.json(res)
                                                                                response
                                                                            }
                                                                                };
            output.extend(update_handler);
        }
        let select = "SELECT ".to_owned() + &cols.to_owned() + " FROM " + &table_name.to_owned();

        let get = "get_".to_owned() + &table_name.to_lowercase();

        let getfunctionname = syn::Ident::new(&get, proc_macro2::Span::call_site());
        let new_function = quote! {

            async fn #getfunctionname (pool: &sqlx::postgres::PgPool) -> Result<Vec<#struct_name>, sqlx::Error> {

        let rows: Vec<#struct_name> =  sqlx::query_as::<_,#struct_name>(#select).fetch_all(pool).await?;

        Ok(rows)


                            }
                        };
        let route = "/".to_owned() + table_name;
        let get_handler_function_name = "get_".to_owned() + &table_name.to_lowercase() + "_handler";
        let get_handler_function_name_syn =
            syn::Ident::new(&get_handler_function_name, proc_macro2::Span::call_site());
        let get_handler = quote! {
        #[get(#route)]
        async fn #get_handler_function_name_syn(pool: web::Data<PgPool>) -> impl Responder {
            let res: Vec<#struct_name> = #getfunctionname(pool.get_ref()).await.unwrap();
            let mut response = HttpResponse::Ok();
            response.insert_header(("Content-Type", "application/json"));
            response.json(res)
        }
            };

        let post_handler_function_name =
            "post_".to_owned() + &table_name.to_lowercase() + "_handler";
        let post_handler_function_name_syn =
            syn::Ident::new(&post_handler_function_name, proc_macro2::Span::call_site());
        let post_handler = quote! {
        #[post(#route)]
        async fn #post_handler_function_name_syn(record: web::Json<#struct_name>, pool: web::Data<PgPool>) -> impl Responder {

                let json = serde_json::to_value(record).unwrap();
                let fields: Vec<&str> = json.as_object().unwrap().keys().map(|s| s.as_str()).collect();
                let placeholders: Vec<String> = (1..=fields.len()).map(|i| format!("${}", i)).collect();
                let values: Vec<&serde_json::Value> = json.as_object().unwrap().values().collect();
                let query = format!( "INSERT INTO {} ({}) VALUES ({})", #table_name, fields.join(", "), placeholders.join(", ") );
                let mut query_builder = sqlx::query(&query);
                for (i, value) in values.iter().enumerate() {
                    query_builder = match value {
                        serde_json::Value::String(s) => query_builder.bind(s),
                        serde_json::Value::Number(n) if n.is_i64() => query_builder.bind(n.as_i64().unwrap()),
                        serde_json::Value::Number(n) if n.is_f64() => query_builder.bind(n.as_f64().unwrap()),
                        serde_json::Value::Bool(b) => query_builder.bind(*b),
                        _ => query_builder,
                    };
                }

                println!("{}", "Processing data");
                query_builder.execute(pool.get_ref()).await;

            let mut response = HttpResponse::Ok();
            response.insert_header(("Content-Type", "application/json"));
                response.json(json)
        }
            };

        output.extend(new_struct);
        output.extend(new_struct2);
        output.extend(new_function);
        output.extend(get_handler);
        output.extend(post_handler);
    }
    println!("{}", output);
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
            #[derive(Deserialize,Debug)]
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
