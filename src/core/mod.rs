use comfy::{Entity, Transform};

use crate::state::Reality;

pub mod position;
pub mod anycellmap;
pub mod animation;

pub trait Initializable {
    fn initialize(&mut self, entity: &Entity, transform: &mut Transform, reality: &mut Reality);

    fn is_initialized(&self) -> bool;
}