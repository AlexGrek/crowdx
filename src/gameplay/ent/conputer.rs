use comfy::{Entity, IVec2, Transform};

use crate::state::Reality;

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
        self.initialized = true
    }
    
    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl MapEntityObject for Conputer {}