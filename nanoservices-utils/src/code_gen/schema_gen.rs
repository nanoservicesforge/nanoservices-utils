use schemars::schema::RootSchema;
use crate::code_gen::configure_paths::prep_file_path;
use std::fs::File;


#[macro_export]
macro_rules! generate_schema_files {
    ($root_path:literal, $( $contract:ident => $file_path:literal ),*) => {{
        let root = $root_path;
        $(
            let schema = schemars::schema_for!($contract);
            nanoservices_utils::code_gen::schema_gen::generate_schema_file(schema, $file_path, root);
        )*
    }};
}



pub fn generate_schema_file(schema: RootSchema, path: &str, root: &str) {
    let file_path = prep_file_path(path, root);
    let file = File::create(file_path).expect("Failed to create file");
    serde_json::to_writer_pretty(file, &schema).expect("Failed to write schema to file");
}
