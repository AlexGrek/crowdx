use std::fs::{self, File};
use std::io::{self, BufRead};

pub fn read_file_lines(path: &str) -> io::Result<Vec<String>> {
    // Open the file in read-only mode with error handling
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    // Iterate over lines and collect them into a vector
    let lines: io::Result<Vec<String>> = reader.lines().collect();
    lines
}

pub fn must_read_file_lines(path: &str) -> Vec<String> {
    read_file_lines(path).unwrap()
}


pub fn list_files_in_directory(dir_path: &str) -> Result<Vec<String>, std::io::Error> {
    let mut files = Vec::new();

    // Read directory entries
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        
        // Check if the entry is a file
        if metadata.is_file() {
            if let Some(file_name) = entry.file_name().to_str() {
                files.push(file_name.to_string());
            }
        }
    }

    Ok(files)
}

pub fn list_assets_subdirectory(dir_path: &str) -> Result<Vec<String>, std::io::Error> {
    let path = env!("CARGO_MANIFEST_DIR").to_owned() + &format!("/assets/{}", { dir_path });
    list_files_in_directory(&path)
}
