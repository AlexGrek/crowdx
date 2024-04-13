use comfy::{Arc, HashMap, Mutex};

use crate::{
    behavior::{
        carriable::carriableitem::CarriableItems, interactive::InteractiveObjects,
        messaging::MessagingHosts,
    }, core::position::Ps, gameplay::gametime::Time, persistence::Persistence, worldmap::Cellmap
};

pub struct Reality {
    pub cellmap: Cellmap,
    pub carriables: Arc<CarriableItems>,
    pub messaging: Arc<MessagingHosts>,
    pub interactive: Arc<InteractiveObjects>,
    pub persistence: Persistence,
    pub time: Time,
}

impl Reality {
    pub fn new(cellmap: Cellmap) -> Self {
        Self {
            cellmap,
            carriables: Arc::new(Mutex::new(HashMap::new())),
            messaging: Arc::new(Mutex::new(HashMap::new())),
            interactive: Arc::new(Mutex::new(HashMap::new())),
            persistence: Persistence::new(),
            time: Time::new(16*60)
        }
    }
}

pub struct WorldState {
    pub reality: Reality,
    pub x: i32,
    pub y: i32,
    pub selected_cell: Ps,
    pub selected: bool,
    pub initialized: bool,
    pub entities_initialized: bool,
    pub dog_order: Option<Ps>,
    pub paused: bool,
}

impl WorldState {
    pub fn select_cell(&mut self, position: Ps) {
        self.selected_cell = position;
        self.selected = true;
    }

    pub fn deselect_cell(&mut self) {
        println!("deselected");
        self.selected = false;
        self.selected_cell = (100500, 100500).into();
    }

    pub fn select_or_deselect_cell(&mut self, position: Ps) {
        if self.selected_cell != position {
            self.select_cell(position)
        } else {
            self.deselect_cell()
        }
    }
}
