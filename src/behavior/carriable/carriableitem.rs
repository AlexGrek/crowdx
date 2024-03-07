use comfy::{Entity, HashMap, Mutex, Vec2, RandomRange};

use crate::{behavior::creatures::PsOffsetProvider, core::position::Ps};

#[derive(Debug, Clone, Copy)]
pub struct CarriableItemHandle {
    pub item_id: Entity,
    pub item_type: &'static str,
    pub carried_by: Option<Entity>,
    pub position: Ps,
    pub offset: Vec2,
    pub personal_offset: Vec2,
    pub consumed: bool
}

impl CarriableItemHandle {
    pub fn new(item_type: &'static str, entity: Entity, position: Ps) -> Self {
        let off = Vec2::new(f32::gen_range(-0.2, 0.2), f32::gen_range(0.01, 0.2));
        Self {
            item_id: entity,
            carried_by: None,
            position,
            consumed: false,
            offset: off,
            personal_offset: off,
            item_type
        }
    }

    pub fn copy_position(&mut self, from: &dyn PsOffsetProvider) {
        self.position = from.get_ps();
        self.offset = from.get_offset() + self.personal_offset;
    }

    pub fn available(&self) -> bool {
        !self.consumed && self.carried_by.is_none()
    }

    pub fn take(&mut self, entity: Entity) {
        self.carried_by = Some(entity)
    }

    pub fn release(&mut self) {
        self.carried_by = None
    }

    pub fn consume(&mut self) {
        self.consumed = true
    }
}

impl PsOffsetProvider for CarriableItemHandle {
    fn get_ps (&self) -> Ps {
        self.position
    }

    fn get_offset (&self) -> comfy::Vec2 {
        self.offset
    }
}

pub type CarriableItems = Mutex<HashMap<Entity,CarriableItemHandle>>;