pub mod conputer;
pub mod bed;
pub mod officeworker;

use std::fmt::Debug;

pub trait MapEntityObject: Debug + Send + Sync {}

#[derive(Debug, Copy, Clone)]
pub struct Grass;

impl MapEntityObject for Grass {}
