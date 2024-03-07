use comfy::Path;

pub trait Flatten<T> {
    fn flatten(self) -> Option<T>;
}

impl<T> Flatten<T> for Option<Option<T>> {
    fn flatten(self) -> Option<T> {
        match self {
            None => None,
            Some(v) => v,
        }
    }
}

pub fn get_file_name(path: &str) -> Option<String> {
    let path = Path::new(path);
    path.file_name().and_then(|file_name| file_name.to_str().map(|s| s.to_string()))
}
