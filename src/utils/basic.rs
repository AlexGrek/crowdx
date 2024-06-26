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

pub fn max_ignore_nan(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() || a.is_infinite() || b.is_infinite() {
        panic!("Encountered NaN or infinity: {},{}", a, b);
    }
    if a > b {
        a
    } else {
        b
    }
}
