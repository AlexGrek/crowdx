pub mod basic;
pub mod anyhashmap;
pub mod fileutils;

pub fn create_vec<T>(size: usize) -> Vec<Option<T>> {
    let mut vec: Vec<Option<T>> = Vec::with_capacity(size);
    vec.resize_with(size, || None);
    vec
}

pub fn create_vec_capacity<T>(size: usize) -> Vec<Option<T>> {
    let vec: Vec<Option<T>> = Vec::with_capacity(size);
    vec
}

pub fn is_string_in_array(string: &str, array: &[&'static str]) -> bool {
    for &element in array {
        if element == string {
            return true;
        }
    }
    false
}