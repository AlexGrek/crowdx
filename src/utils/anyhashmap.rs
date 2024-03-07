use comfy::{Arc, HashMap, Mutex};

// Type alias for the hashmap
pub type PrimitiveHashMap = Arc<Mutex<HashMap<String, PrimitiveValue>>>;

// Define an enum to represent primitive values

#[derive(Debug, Clone, Copy)]
pub enum PrimitiveValue {
    Integer(i32),
    Float(f64),
    Boolean(bool),
    Character(char),
}

// Function to create and return the hashmap
pub fn create_primitive_hashmap() -> PrimitiveHashMap {
    Arc::new(Mutex::new(HashMap::new()))
}

// Function to insert a value into the hashmap
pub fn insert_value(map: &PrimitiveHashMap, key: &str, value: PrimitiveValue) {
    map.lock().insert(key.to_string(), value);
}

// Function to get a value from the hashmap and downcast it to its actual type
pub fn get_value(map: &PrimitiveHashMap, key: &str) -> Option<PrimitiveValue> {
    map.lock().get(key).cloned()
}
