use comfy::{Entity, IVec2, Transform};

use crate::{behavior::interactive::InteractiveObjectHandle, core::position::Ps, state::Reality};

use super::MapEntityObject;

#[derive(Debug, Copy, Clone)]
pub struct Conputer {
    pub workplace: IVec2,
    pub initialized: bool,
    pub use_animation_playing: bool,
    pub handle_position: Ps,
}

impl Conputer {
    pub fn new(workplace: IVec2) -> Conputer {
        Conputer {
            workplace,
            initialized: false,
            use_animation_playing: true,
            handle_position: Ps {x: 0, y: 0}
        }
    }
}

impl crate::core::Initializable for Conputer {
    fn initialize(&mut self, entity: &Entity, transform: &mut Transform, reality: &mut Reality) {
        self.initialized = true;
        // println!("Conputer {:?} initialized:: {:?}", entity, transform);
        let mut lock = reality.interactive.lock();
        let ps: Ps = transform.position.into();
        let handle = InteractiveObjectHandle::new(
            crate::behavior::item_types::CONPUTER,
            entity.to_owned(),
            ps,
            Some(self.workplace),
        );
        self.handle_position = handle.get_interactive_ps();
        lock.insert(
            self.handle_position,
            handle,
        );
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl MapEntityObject for Conputer {}
