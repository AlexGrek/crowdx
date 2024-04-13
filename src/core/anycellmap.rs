use comfy::num_traits::ToPrimitive;

use super::position::{Ps, XYprovider};

pub struct AnyCellmap<T: Clone> {
    pub map: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> AnyCellmap<T> where T: Clone {
    pub fn new(init_value: &T, width: i32, height: i32) -> Self {
        let width = width.to_usize().unwrap();
        let height = height.to_usize().unwrap();
        Self {
            map: vec![init_value.clone(); width * height],
            width, height
        }
    }

    pub fn reset_and_resize(&mut self, init_value: T, width: i32, height: i32) {
        let width = width.to_usize().unwrap();
        let height = height.to_usize().unwrap();
        self.width = width;
        self.height = height;
        self.map = vec![init_value; width * height]
    }

    pub fn reset(&mut self, init_value: T) {
        self.map = vec![init_value; self.width * self.height]
    }

    pub fn wh_usize(&self) -> (usize, usize) {
        return (self.width, self.height);
    }

    pub fn get_xy<C>(&self, x: C, y: C) -> &T
    where
        C: TryInto<usize> + Copy,
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

    pub fn get_pos(&self, pos_provider: &dyn XYprovider) -> &T {
        let pos = pos_provider.get_xy();
        return self.get_xy(pos.0, pos.1);
    }

    pub fn get_pos_mut(&mut self, pos_provider: &dyn XYprovider) -> &mut T {
        let pos = pos_provider.get_xy();
        return self.get_xy_mut(pos.0, pos.1);
    }

    pub fn within_bounds<C>(&self, x: C, y: C) -> bool
    where
        C: TryInto<i32> + Copy,
    {
        let numx: i32 = x.try_into().unwrap_or(0);
        let numy: i32 = y.try_into().unwrap_or(0);

        return numx >= 0 && numy >= 0 && numx < (self.width as i32) && numy < (self.height as i32);
    }

    pub fn get_xy_mut<C>(&mut self, x: C, y: C) -> &mut T
    where
        C: TryInto<usize> + Copy,
    {
        let numx: usize = x.try_into().unwrap_or(0);
        let numy: usize = y.try_into().unwrap_or(0);

        &mut self.map[numy * self.width + numx]
    }
}
