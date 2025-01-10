extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse::Parse, parse::ParseStream,
    ItemFn, Ident, Token, LitStr,
};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::TokenTree::Punct;
use proc_macro2::TokenTree;
use syn::{ItemStruct, Fields, Type, DeriveInput};
use std::collections::HashMap;
mod typescript_gen;


fn is_http_method_allowed(method: &str) -> bool {
    matches!(
        method,
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" | "CONNECT" | "TRACE"
    )
}

fn is_framework_allowed(framework: &str) -> bool {
    matches!(framework, "ROCKET" | "ACTIX" | "AXUM")
}

#[proc_macro_attribute]
pub fn code_gen_api_endpoint(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args: TokenStream2 = parse_macro_input!(attr);
    let args_ref = args.clone();

    let mut buffer = Vec::new();
    let mut cache = Vec::new();

    for arg in args {
        match arg  {
            Punct(ref punct) => {
                if punct.as_char() == ',' {
                    buffer.push(cache);
                    cache = Vec::new();
                }
                else {
                    cache.push(arg);
                }
            },
            _ => {
                cache.push(arg);
            }
        }
    }

    let mut uri: Option<String> = None;
    let mut method: Option<String> = None;
    let mut framework: Option<String> = None;
    let mut input_schema_path: Option<String> = None;
    let mut output_schema_path: Option<String> = None;
    let mut output_typescript_path: Option<String> = None;
    let mut expected_response_code: Option<u16> = None;
    for param in buffer {

        if param.len() < 3 {
            panic!("All parameters must be in the form `key=value`");
        }

        match &param[1] {
            TokenTree::Punct(punct) => {
                if punct.as_char().to_string().as_str() != "=" {
                    panic!("All parameters must be in the form `key=value`");
                }
            },
            _ => {
                panic!("All parameters must be in the form `key=value`");
            }
        }

        match &param[0] {
            TokenTree::Ident(ident) => {
                let param_name = ident.to_string();
                match param_name.as_str() {
                    "uri" => {
                        uri = Some(param[2].to_string().replace("\"", ""));
                    },
                    "method" => {
                        let method_str = param[2].to_string().replace("\"", "").to_uppercase();
                        let allowed = is_http_method_allowed(&method_str);
                        if !allowed {
                            panic!("Invalid HTTP method: {}", method_str);
                        }
                        method = Some(method_str);
                    },
                    "framework" => {
                        let framework_str = param[2].to_string().replace("\"", "").to_uppercase();
                        let allowed = is_framework_allowed(&framework_str);
                        if !allowed {
                            panic!("Invalid framework: {}", framework_str);
                        }
                        framework = Some(framework_str);
                    },
                    "additional_headers" => {
                        // for i in param[2] {
                        //     eprint!("\n\nheader data {:?}\n\n", i);
                        // }
                        match &param[2] {
                            proc_macro2::TokenTree::Group(unwraped_headers) => {
                                for i in unwraped_headers.stream() {
                                    match i {
                                        TokenTree::Literal(literal) => {
                                            eprintln!("main deal{:?}", literal);
                                        },
                                        _ => {
                                            eprintln!("{:?}", i);
                                        }
                                    }
                                    // eprintln!("{:?}", i);
                                }
                            },
                            _ => {
                                panic!("headers should be a vector of header strings");
                            }
                        }
                        // eprint!("\n\nheader data {:?}\n\n", param[2]);
                    },
                    "input_schema" => {
                        input_schema_path = Some(param[2].to_string().replace("\"", ""));
                    },
                    "output_schema" => {
                        output_schema_path = Some(param[2].to_string().replace("\"", ""));
                    },
                    "output_typescript_path" => {
                        output_typescript_path = Some(param[2].to_string().replace("\"", ""));
                    }
                    "expected_response_code" => {
                        let response_code = param[2].to_string().replace("\"", "");
                        expected_response_code = Some(response_code.parse::<u16>().expect("Invalid response code"));

                    },
                    _ => {
                        eprint!("\n\n{:?}\n\n", param[0]);
                    }
                }
            },
            _ => {
                eprint!("\n\n{:?}\n\n", param[0]);
            }
        }
        // eprint!("\n\n{:?}\n\n", param[0].as_char());
    }
    eprint!("\n\nuri {:?}\n\n", uri);
    eprint!("\n\nmethod {:?}\n\n", method);
    eprint!("\n\nframework {:?}\n\n", framework);
    eprint!("\n\ninput_schema_path {:?}\n\n", input_schema_path);
    eprint!("\n\noutput_schema_path {:?}\n\n", output_schema_path);
    eprint!("\n\noutput_typescript_path {:?}\n\n", output_typescript_path);

    let uri = uri.expect("uri parameter is required");
    let method = method.expect("method parameter is required");
    let expected_response_code = expected_response_code.expect("expected_response_code parameter is required");
    let additional_headers: Vec<String> = vec![];

    // Parse the input function
    let input_fn = parse_macro_input!(item as ItemFn);

    let mut body: Option<Ident> = None;

    // Extract the inner type of the Json<> input parameter
    for input in input_fn.sig.inputs.iter() {
        if let syn::FnArg::Typed(arg) = input {
            if let syn::Type::Path(syn::TypePath { path, .. }) = &*arg.ty {
                if path.segments.first().unwrap().ident == "Json" {
                    // Extract inner type (e.g., NewToDoItem)
                    if let syn::PathArguments::AngleBracketed(args) = &path.segments.last().unwrap().arguments {
                        if let Some(syn::GenericArgument::Type(syn::Type::Path(inner))) = args.args.first() {
                            body = Some(inner.path.segments.first().unwrap().ident.clone());
                            break
                        }
                    }
                }
            }
        }
    }

    let typescript_function = typescript_gen::generate_typescript(
        input_fn.sig.ident.to_string().as_str(),
        &uri,
        &method,
        vec![],
        expected_response_code,
        input_schema_path,
        output_schema_path,
    );

    eprintln!("{}", typescript_function);

    // Generate expanded code
    let expanded = quote! {
        // #[allow(non_upper_case_globals)]
        // const API_URI: &str = #uri;

        // #[allow(non_upper_case_globals)]
        // const API_METHOD: &str = #method;

        #input_fn
    };

    TokenStream::from(expanded)
}



/// A recursive function to extract schema from a struct.
fn extract_schema_recursively(
    ident: &syn::Ident,
    available_items: &HashMap<String, syn::ItemStruct>,
) -> HashMap<String, serde_json::Value> {
    let mut schema = HashMap::new();

    // Look up the struct by name in the parsed items
    if let Some(struct_item) = available_items.get(&ident.to_string()) {
        if let Fields::Named(fields) = &struct_item.fields {
            for field in &fields.named {
                let field_name = field.ident.as_ref().unwrap().to_string();
                match &field.ty {
                    Type::Path(type_path) => {
                        let type_name = type_path.path.segments.last().unwrap().ident.to_string();

                        if available_items.contains_key(&type_name) {
                            // If it's a nested struct, recursively extract its schema
                            let nested_schema = extract_schema_recursively(
                                &type_path.path.segments.last().unwrap().ident,
                                available_items,
                            );
                            schema.insert(field_name, serde_json::json!(nested_schema));
                        } else {
                            // Otherwise, treat it as a primitive or external type
                            schema.insert(field_name, serde_json::json!(type_name));
                        }
                    }
                    _ => {
                        schema.insert(field_name, serde_json::json!("unknown"));
                    }
                }
            }
        }
    }

    schema
}

/// Parses the entire file and builds a map of all struct definitions.
fn parse_structs_in_file(input: &proc_macro2::TokenStream) -> HashMap<String, syn::ItemStruct> {
    let mut structs = HashMap::new();

    let syntax_tree: syn::File = syn::parse2(input.clone()).expect("Failed to parse input file");

    for item in syntax_tree.items {
        if let syn::Item::Struct(item_struct) = item {
            structs.insert(item_struct.ident.to_string(), item_struct);
        }
    }

    structs
}