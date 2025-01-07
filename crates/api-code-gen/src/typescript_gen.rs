use serde_json::Value;
use serde_json::Map;
use std::fs::File;
use std::io::Read;
use std::io::BufReader;


pub fn generate_typescript(
    function_name: &str,
    uri: &str,
    method: &str,
    additional_headers: Vec<String>,
    expected_response_code: u16,
    input_schema_path: Option<String>,
    output_schema_path: Option<String>,
) -> String {
    let mut ts_code = String::new();
    let mut incoming_type: Option<String> = None;
    let mut outgoing_type: Option<String> = None;

    eprintln!(
        "Comparing output_schema_path: {:?} with input_schema_path: {:?}",
        output_schema_path, input_schema_path
    );

    // Add imports
    ts_code.push_str(&generate_typescript_imports());

    let input_schema = match input_schema_path.clone() {
        Some(path) => {
            let file = File::open(path).expect("Failed to open input schema file");
            let mut reader = BufReader::new(file);
            let mut contents = String::new();
            reader.read_to_string(&mut contents).expect("Failed to read input schema file");
            let json_value: Value = serde_json::from_str(&contents).expect("Failed to parse input schema JSON");
            Some(json_value)
        },
        None => {
            None
        }
    };
    let output_schema = match output_schema_path.clone() {
        Some(path) => {
            let file = File::open(path).expect("Failed to open output schema file");
            let mut reader = BufReader::new(file);
            let mut contents = String::new();
            reader.read_to_string(&mut contents).expect("Failed to read output schema file");
            let json_value: Value = serde_json::from_str(&contents).expect("Failed to parse input schema JSON");
            Some(json_value)
        },
        None => {
            None
        }
    };
    eprint!("schemas loaded");

    let mut duplicates_registered = false;
    if output_schema_path.is_some() && input_schema_path.is_some() {
        // both the same file so just define it once
        if output_schema_path.unwrap() == input_schema_path.unwrap() {
            ts_code.push_str(json_schema_to_typescript(&output_schema.clone().unwrap()).unwrap().as_str());
            incoming_type = Some(output_schema.clone().unwrap().get("title").unwrap().as_str().unwrap().to_string());
            outgoing_type = Some(input_schema.clone().unwrap().get("title").unwrap().as_str().unwrap().to_string());
            duplicates_registered = true;
        }
    }

    // generate the types for the input and output schemas
    if duplicates_registered == false {
        match input_schema {
            Some(schema) => {
                ts_code.push_str(json_schema_to_typescript(&schema).unwrap().as_str());
                incoming_type = Some(schema.get("title").unwrap().as_str().unwrap().to_string());
            },
            None => {}
        }
        match output_schema {
            Some(schema) => {
                ts_code.push_str(json_schema_to_typescript(&schema).unwrap().as_str());
                outgoing_type = Some(schema.get("title").unwrap().as_str().unwrap().to_string());
            },
            None => {}
        }
    }

    eprintln!("incoming_type: {:?}", incoming_type);
    eprintln!("outgoing_type: {:?}", outgoing_type);

    // Add the Axios function
    ts_code.push_str(&generate_axios_function(
        function_name,
        uri,
        method,
        incoming_type,
        outgoing_type,
        additional_headers,
        expected_response_code,
    ));

    ts_code
}



fn generate_typescript_imports() -> String {
    let mut imports = String::new();
    // Add Axios import
    imports.push_str("import axios from 'axios';\n");
    imports.push_str("\n");
    imports
}



pub fn json_schema_to_typescript(parsed_schema: &Value) -> Result<String, String> {
    // let parsed_schema: Value = serde_json::from_str(schema).map_err(|e| format!("Failed to parse schema: {}", e))?;

    let mut ts_interfaces = String::new();

    if let Some(title) = parsed_schema["title"].as_str() {
        ts_interfaces.push_str(&generate_interface(&parsed_schema, title)?);
    }

    if let Some(definitions) = parsed_schema["definitions"].as_object() {
        for (name, definition) in definitions {
            ts_interfaces.push_str(&generate_interface(definition, name)?);
        }
    }

    Ok(ts_interfaces)
}

fn generate_interface(schema: &Value, name: &str) -> Result<String, String> {
    let mut interface = format!("export interface {} {{\n", name);

    if let Some(properties) = schema["properties"].as_object() {
        for (property_name, property_value) in properties {
            let ts_type = json_type_to_ts_type(property_value)?;
            interface.push_str(&format!("  {}: {};\n", property_name, ts_type));
        }
    }

    interface.push_str("}\n\n");
    Ok(interface)
}

fn json_type_to_ts_type(property_value: &Value) -> Result<String, String> {
    match property_value.get("type").and_then(|t| t.as_str()) {
        Some("integer") => Ok("number".to_string()),
        Some("string") => Ok("string".to_string()),
        Some("object") => {
            if let Some(ref_) = property_value.get("$ref").and_then(|r| r.as_str()) {
                // Extract the referenced type name from "$ref"
                if let Some(type_name) = ref_.split('/').last() {
                    return Ok(type_name.to_string());
                }
            }
            Ok("object".to_string()) // Fallback for generic objects
        }
        Some(other) => Err(format!("Unsupported type: {}", other)),
        None => {
            if let Some(ref_) = property_value.get("$ref").and_then(|r| r.as_str()) {
                // Handle $ref for referenced types
                if let Some(type_name) = ref_.split('/').last() {
                    return Ok(type_name.to_string());
                }
            }
            Err(format!("Unsupported type None for: {:?}", property_value))
        }
    }
}



fn generate_axios_function(
    function_name: &str,
    uri: &str,
    method: &str,
    incoming_type: Option<String>,
    outgoing_type: Option<String>,
    additional_headers: Vec<String>,
    expected_response_code: u16,
) -> String {
    // Format the function signature
    let function_signature = if let Some(incoming) = &incoming_type {
        format!(
            "{}(host: string, body: {}, {})",
            function_name,
            incoming,
            additional_headers
                .iter()
                .map(|h| format!("{}: string", h.to_lowercase().replace('-', "_")))
                .collect::<Vec<_>>()
                .join(", ")
        )
    } else {
        format!(
            "{}(host: string, {})",
            function_name,
            additional_headers
                .iter()
                .map(|h| format!("{}: string", h.to_lowercase().replace('-', "_")))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    // Handle the Axios body parameter
    let data_assignment = if incoming_type.is_some() {
        "data: body,"
    } else {
        ""
    };

    // Handle the return type
    let return_type = outgoing_type.unwrap_or_else(|| "Record<string, any>".to_string());

    // Generate the full Axios function
    format!(
        r#"export async function {}: Promise<{}> {{
            const url = `${{host}}{}`;
            const headers = {{
                "Content-Type": "application/json",
                {}
            }};
            const response = await axios({{
                url,
                method: '{}',
                headers,
                {}
            }});
            if (response.status !== {}) {{
                throw new Error(`Unexpected status code: ${{response.status}}`);
            }}
            return response.data || {{}};
        }}"#,
        function_signature,
        return_type,
        uri,
        additional_headers
            .iter()
            .map(|h| format!(r#""{}": {}"#, h, h.to_lowercase().replace('-', "_")))
            .collect::<Vec<_>>()
            .join(",\n"),
        method.to_uppercase(),
        data_assignment,
        expected_response_code
    )
}

