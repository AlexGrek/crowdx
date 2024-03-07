use comfy::{Entity, HashMap, Mutex};

#[derive(Debug)]
pub struct MessagingHost {}

impl MessagingHost {
    pub fn new() -> Self {
        Self {}
    }
}

pub type MessagingHosts = Mutex<HashMap<Entity, MessagingHost>>;
