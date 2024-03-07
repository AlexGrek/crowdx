use comfy::{Entity, HashMap, IVec2, Mutex};

use crate::{behavior::creatures::PsOffsetProvider, core::position::Ps};

#[derive(Debug, Clone, Copy)]
pub struct InteractiveObjectHandle {
    pub item_id: Entity,
    pub item_type: &'static str,
    pub used_by: Option<Entity>,
    pub position: Ps,
    pub interaction_ps_offset: Option<IVec2>,
    pub assigned: bool
}

impl InteractiveObjectHandle {
    pub fn new(item_type: &'static str, entity: Entity, position: Ps, interaction_offset: Option<IVec2>) -> Self {
        Self {
            item_id: entity,
            used_by: None,
            position,
            assigned: false,
            item_type,
            interaction_ps_offset: interaction_offset
        }
    }

    pub fn available(&self) -> bool {
        !self.assigned && self.used_by.is_none()
    }

    pub fn get_interactive_ps(&self) -> Ps {
        match self.interaction_ps_offset {
            Some(offset) => (self.position + offset).into(),
            None => self.position
        }
    }

    pub fn take(&mut self, entity: Entity) {
        self.used_by = Some(entity)
    }

    pub fn release(&mut self) {
        self.used_by = None
    }

    pub fn assign(&mut self) {
        self.assigned = true
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