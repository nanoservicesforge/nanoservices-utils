use std::path::PathBuf;


/// Preps paths for directory creation so we can store code generated files.
/// 
/// # Arguments
/// * `path` - The path to the file being generated.
/// * `root` - The root directory where all the generated files will be stored.
/// 
/// # Returns
/// A `PathBuf` object that represents the path to the file being generated.
pub fn prep_file_path(path: &str, root: &str) -> PathBuf {
    let path_buf = path.split("/").collect::<Vec<&str>>();
    let root_buf = root.split("/").collect::<Vec<&str>>();

    let mut path = std::path::PathBuf::new();
    
    for p in root_buf {
        path = path.join(p);
    }
    if !path.exists() {
        std::fs::create_dir(&path).expect("Failed to create directory");
    }

    let mut ptr = 0;
    let end_ptr = path_buf.len() - 1;

    for p in &path_buf {
        path = path.join(p);
        if ptr != end_ptr && !path.exists() {
            std::fs::create_dir(&path).expect("Failed to create directory");
        }
        ptr += 1;
    }
    path
}