use comfy::{Entity, IVec2, Transform};

use crate::{behavior::interactive::InteractiveObjectHandle, state::Reality};

use super::MapEntityObject;


#[derive(Debug, Copy, Clone)]
pub struct Conputer {
    pub workplace: IVec2,
    pub initialized: bool,
}

impl Conputer {
    pub fn new(workplace: IVec2) -> Conputer {
        Conputer {
            workplace,
            initialized: false,
        }
    }
}

impl crate::core::Initializable for Conputer {
    fn initialize(&mut self, entity: &Entity, transform: &mut Transform, reality: &mut Reality) {
        self.initialized = true;
        println!("Conputer {:?} initialized:: {:?}", entity, transform);
        let mut lock = reality.interactive.lock();
        lock.insert(entity.to_owned(), InteractiveObjectHandle::new(crate::behavior::item_types::CONPUTER, entity.to_owned(), transform.position.into(), Some(self.workplace)));

    }
    
    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl MapEntityObject for Conputer {}