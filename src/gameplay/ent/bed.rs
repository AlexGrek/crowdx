use comfy::{Entity, Transform};

use crate::{behavior::interactive::InteractiveObjectHandle, state::Reality};

use super::MapEntityObject;


#[derive(Debug, Copy, Clone)]
pub struct Bed {
    pub initialized: bool,
}

impl Bed {
    pub fn new() -> Bed {
        Bed {
            initialized: false,
        }
    }
}

impl crate::core::Initializable for Bed {
    fn initialize(&mut self, entity: &Entity, transform: &mut Transform, reality: &mut Reality) {
        self.initialized = true;
        println!("Bed {:?} initialized", entity);
        let mut lock = reality.interactive.lock();
        lock.insert(entity.to_owned(), InteractiveObjectHandle::new(crate::behavior::item_types::BED, entity.to_owned(), transform.position.into(), None));
    }
    
    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl MapEntityObject for Bed {}