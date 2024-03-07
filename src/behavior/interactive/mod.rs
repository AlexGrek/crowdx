use comfy::{Entity, HashMap, Mutex};

use crate::{behavior::creatures::PsOffsetProvider, core::position::Ps};

#[derive(Debug, Clone, Copy)]
pub struct InteractiveObjectHandle {
    pub item_id: Entity,
    pub item_type: &'static str,
    pub used_by: Option<Entity>,
    pub position: Ps,
    pub consumed: bool
}

impl InteractiveObjectHandle {
    pub fn new(item_type: &'static str, entity: Entity, position: Ps) -> Self {
        Self {
            item_id: entity,
            used_by: None,
            position,
            consumed: false,
            item_type
        }
    }

    pub fn available(&self) -> bool {
        !self.consumed && self.used_by.is_none()
    }

    pub fn take(&mut self, entity: Entity) {
        self.used_by = Some(entity)
    }

    pub fn release(&mut self) {
        self.used_by = None
    }

    pub fn consume(&mut self) {
        self.consumed = true
    }
}

impl PsOffsetProvider for InteractiveObjectHandle {
    fn get_ps (&self) -> Ps {
        self.position
    }

    fn get_offset (&self) -> comfy::Vec2 {
        comfy::vec2(0.0, 0.0)
    }
}

pub type InteractiveObjects = Mutex<HashMap<Entity,InteractiveObjectHandle>>;