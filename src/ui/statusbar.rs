use indexmap::IndexMap;

pub struct Statusbar {
    pub bars: IndexMap<String, Bar>,
}


pub struct Bar {
    pub value: f32,
    pub max: f32
}

impl Statusbar {
    pub fn new() -> Self {
        Statusbar {
            bars: IndexMap::new(),
        }
    }

    pub fn show(&mut self, name: String, value: f32, max: f32) {
        if !self.bars.contains_key(&name) {
            self.bars.insert(name, Bar::new(value, max));
        } else {
            let item = self.bars.get_mut(&name).unwrap();
            item.update(value, max);
        }
    }
}

impl Bar {
    pub fn new(value: f32, max: f32) -> Self {
        Bar {
            value,
            max,
        }
    }

    pub fn update(&mut self, value: f32, max: f32) {
        self.max = max;
        self.value = value;
    }

    pub fn normalized(&self) -> f32 {
        let value = self.value / self.max;
        if value < 0.0 {
            return 0.0;
        }
        if value > 1.0 {
            return 1.0;
        }
        return value;
    }
}