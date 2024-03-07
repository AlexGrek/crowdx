use crate::core::animation::BasicTileAnimation;
use crate::core::position::{Ps, XYprovider};
use comfy::{num_traits::ToPrimitive, Itertools};
use comfy::{HashMap, IVec2, RandomRange};
use tiled::PropertyValue;

#[derive(Debug)]
pub struct TileReference {
    pub tile_index: u32,
    pub klass: String,
    pub tile_name: String,
    pub tile_image: String,
    pub props: HashMap<String, PropertyValue>,
    pub size: IVec2,
    pub animated: Option<BasicTileAnimation>,
}

impl TileReference {
    pub fn extract_bool_value(property_value: &PropertyValue) -> Option<bool> {
        if let PropertyValue::BoolValue(value) = property_value {
            Some(*value)
        } else {
            None
        }
    }

    // Function to extract a floating point value from a PropertyValue
    pub fn extract_float_value(property_value: &PropertyValue) -> Option<f32> {
        if let PropertyValue::FloatValue(value) = property_value {
            Some(*value)
        } else {
            None
        }
    }

    // Function to extract an integer value from a PropertyValue
    pub fn extract_int_value(property_value: &PropertyValue) -> Option<i32> {
        if let PropertyValue::IntValue(value) = property_value {
            Some(*value)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct CellStatus {
    pub occupied: bool,
}

#[derive(Debug)]
pub struct Cell {
    pub passable: bool,
    pub reference: Option<TileReference>,
    pub position: Ps,
    pub status: CellStatus,
}

impl CellStatus {
    fn new() -> CellStatus {
        return CellStatus { occupied: false };
    }
}

impl Cell {
    pub fn new(position: (i32, i32), passable: bool, reference: Option<TileReference>) -> Cell {
        Cell {
            passable: passable,
            reference: reference,
            position: position.into(),
            status: CellStatus::new(),
        }
    }

    pub fn occupy(&mut self) {
        self.status.occupied = true;
    }

    pub fn deoccupy(&mut self) {
        self.status.occupied = false;
    }

    pub fn is_passable(&self, concern_occupied: bool) -> bool {
        if concern_occupied {
            return self.passable && !self.status.occupied;
        } else {
            return self.passable;
        }
    }

    pub fn get_tile_name(&self) -> Option<String> {
        match &self.reference {
            Some(refer) => Some(refer.tile_image.to_string()),
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct Cellmap {
    pub map: Vec<Cell>,
    width: usize,
    height: usize,
}

type DumpCellClosure = dyn Fn(&Cell) -> String;

impl Cellmap {
    pub fn new(vec: Vec<Option<Cell>>, width: i32, height: i32) -> Cellmap {
        Cellmap {
            map: vec.into_iter().map(|x| x.unwrap()).collect_vec(),
            width: width.to_usize().unwrap(),
            height: height.to_usize().unwrap(),
        }
    }

    pub fn occupy_ps(&mut self, pos: &Ps) {
        self.get_pos_mut(pos).occupy();
    }

    pub fn deoccupy_ps(&mut self, pos: &Ps) {
        self.get_pos_mut(pos).deoccupy();
    }

    pub fn move_occupy_ps(&mut self, pos_from: &Ps, pos_to: &Ps) {
        self.get_pos_mut(pos_from).deoccupy();
        self.get_pos_mut(pos_to).occupy();
    }

    pub fn occupy_xy<T>(&mut self, x: T, y: T)
    where
        T: TryInto<usize> + Copy,
    {
        self.get_xy_mut(x, y).occupy()
    }

    pub fn deoccupy_xy<T>(&mut self, x: T, y: T)
    where
        T: TryInto<usize> + Copy,
    {
        self.get_xy_mut(x, y).deoccupy()
    }

    pub fn move_occupy_xy<T>(&mut self, x: T, y: T, x1: T, y1: T)
    where
        T: TryInto<usize> + Copy,
    {
        self.get_xy_mut(x1, y1).occupy();
        self.get_xy_mut(x, y).deoccupy();
    }

    pub fn get_xy<T>(&self, x: T, y: T) -> &Cell
    where
        T: TryInto<usize> + Copy,
    {
        let numx: usize = x.try_into().unwrap_or(0);
        let numy: usize = y.try_into().unwrap_or(0);

        &self.map[numy * self.width + numx]
    }

    pub fn pos_within_bounds(&self, pos: Ps) -> bool {
        return self.within_bounds(pos.x, pos.y);
    }

    pub fn xy_within_bounds(&self, pos_provider: &dyn XYprovider) -> bool {
        let pos = pos_provider.get_xy();
        return self.within_bounds(pos.0, pos.1);
    }

    pub fn get_pos(&self, pos_provider: &dyn XYprovider) -> &Cell {
        let pos = pos_provider.get_xy();
        return self.get_xy(pos.0, pos.1);
    }

    pub fn get_pos_mut(&mut self, pos_provider: &dyn XYprovider) -> &mut Cell {
        let pos = pos_provider.get_xy();
        return self.get_xy_mut(pos.0, pos.1);
    }

    pub fn within_bounds<T>(&self, x: T, y: T) -> bool
    where
        T: TryInto<i32> + Copy,
    {
        let numx: i32 = x.try_into().unwrap_or(0);
        let numy: i32 = y.try_into().unwrap_or(0);

        return numx >= 0 && numy >= 0 && numx < (self.width as i32) && numy < (self.height as i32);
    }

    pub fn get_xy_mut<T>(&mut self, x: T, y: T) -> &mut Cell
    where
        T: TryInto<usize> + Copy,
    {
        let numx: usize = x.try_into().unwrap_or(0);
        let numy: usize = y.try_into().unwrap_or(0);

        &mut self.map[numy * self.width + numx]
    }

    pub fn wh_i32(&self) -> (i32, i32) {
        return (self.width.to_i32().unwrap(), self.height.to_i32().unwrap());
    }

    pub fn wh_u32(&self) -> (u32, u32) {
        return (self.width.to_u32().unwrap(), self.height.to_u32().unwrap());
    }

    pub fn wh_usize(&self) -> (usize, usize) {
        return (self.width, self.height);
    }

    pub fn pick_random_passable_ps(&self) -> Ps {
        // UNSAFE!!! Not guaranteed to ever finish
        loop {
            let x = usize::gen_range(0, self.width);
            let y = usize::gen_range(0, self.height);
            if self.get_xy(x, y).is_passable(true) {
                return Ps { x, y };
            }
        }
    }

    pub fn print(&self, dump_cell_closure: &DumpCellClosure) {
        let mut counter = 0;
        for i in 0..self.map.len() {
            let item = &self.map[i];
            let text = format!("{:width$}", dump_cell_closure(item), width = 2);
            print!("{}", text);
            counter += 1;
            if counter == self.width {
                println!("");
                counter = 0;
            }
        }
    }
}
