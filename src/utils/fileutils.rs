use std::fs::File;
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