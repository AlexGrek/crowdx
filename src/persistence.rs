// persistence is a way for GameObjects to know anything about what is around

use comfy::{HashMap, Itertools, Vec2};

use crate::core::position::Ps;

pub struct PersistentCell {
    item_class: &'static str,
    item_subclass: &'static str,
    offset: Vec2,
}

pub struct Persistence {
    map: HashMap<Ps, HashMap<&'static str, PersistentCell>>,
}

impl Persistence {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn map_cell<F, T>(&self, ps: Ps, fnct: F) -> Option<T>
    where
        F: Fn(&HashMap<&str, PersistentCell>) -> T,
    {
        self.map.get(&ps).map(|borrowed| fnct(borrowed))
    }

    pub fn remove_class(&mut self, ps: Ps, class_name: &str) -> Option<PersistentCell> {
        let map = self.map.get_mut(&ps);
        map.map(|f| f.remove(class_name)).flatten()
    }

    pub fn put_class(&mut self, ps: Ps, item: PersistentCell) {
        let exists = self.map.contains_key(&ps);
        if !exists {
            self.map.insert(ps, HashMap::new());
        }
        self.map.get_mut(&ps).unwrap().insert(item.item_class, item);
    }

    pub fn get_all_classes(&mut self, ps: Ps) -> Vec<&'static str> {
        self.map
            .get(&ps)
            .map(|data| {
                data.keys()
                    .into_iter()
                    .map(|slink| slink.to_owned())
                    .collect_vec()
            })
            .unwrap_or(Vec::new())
    }
}
