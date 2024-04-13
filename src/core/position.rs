use core::fmt;

use comfy::{num_traits::ToPrimitive, vec2, IVec2, Vec2};

#[derive(Copy, Clone, PartialEq, Eq, comfy::Hash)]
pub struct Ps {
    pub x: usize,
    pub y: usize,
}

impl From<(i32, i32)> for Ps {
    fn from(value: (i32, i32)) -> Self {
        Self {
            x: value.0.to_usize().unwrap(),
            y: value.1.to_usize().unwrap(),
        }
    }
}

impl fmt::Debug for Ps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Customize the output format
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl From<Vec2> for Ps {
    fn from(value: Vec2) -> Self {
        Self {
            x: value.x.floor().to_usize().unwrap(),
            y: value.y.floor().to_usize().unwrap(),
        }
    }
}

impl From<(usize, usize)> for Ps {
    fn from(value: (usize, usize)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl Into<comfy::Vec2> for Ps {
    fn into(self) -> comfy::Vec2 {
        comfy::Vec2::new(self.x.to_f32().unwrap(), self.y.to_f32().unwrap())
    }
}

impl Into<PsSigned> for Ps {
    fn into(self) -> PsSigned {
        PsSigned {
            x: self.x as isize,
            y: self.y as isize,
        }
    }
}

impl Ps {
    pub fn manhattan_distance(&self, other: &Ps) -> i32 {
        let diffx = self.x.abs_diff(other.x);
        let diffy = self.y.abs_diff(other.y);
        return (diffx + diffy).to_i32().unwrap();
    }

    pub fn distance_to(&self, other: &dyn PsProvider) -> Vec2 {
        let diffx = other.get_current_ps().x as f32 - self.x as f32;
        let diffy = other.get_current_ps().y as f32 - self.y as f32;
        return vec2(diffx, diffy);
    }

    pub fn distance_to_normalize(&self, other: &dyn PsProvider) -> Vec2 {
        let distance = self.distance_to(other);
        let normal = crate::utils::basic::max_ignore_nan(distance[1].abs(), distance[0].abs());
        let normalized = vec2(distance[0] /  normal, distance[1] / normal);
        return normalized;
    }

    pub fn with_normalized_change(&self, change: Vec2, threshold: f32) -> Ps {
        let mut copy = self.clone();
        // x
        if change[0].abs() > threshold {
            let sign = change[0].signum() as isize;
            copy.x = (sign + copy.x as isize) as usize;
        }
        // y
        if change[1].abs() > threshold {
            let sign = change[1].signum() as isize;
            copy.y = (sign + copy.y as isize) as usize;
        }
        if self.eq(&copy) {
            panic!("Same ps produced by {:?}.with_normalized_change({:?}, {})", self, change, threshold);
        }
        return copy;
    }
}

impl std::ops::Add for Ps {
    type Output = PsSigned;

    fn add(self, rhs: Self) -> PsSigned {
        let x = self.x as isize - rhs.x as isize;
        let y = self.y as isize - rhs.y as isize;
        return PsSigned { x: x, y: y };
    }
}

impl std::ops::Add::<IVec2> for Ps {
    type Output = PsSigned;

    fn add(self, rhs: IVec2) -> PsSigned {
        let x = self.x as isize - rhs.x as isize;
        let y = self.y as isize - rhs.y as isize;
        return PsSigned { x: x, y: y };
    }
}

impl std::ops::Add<PsSigned> for Ps {
    type Output = PsSigned;

    fn add(self, rhs: PsSigned) -> PsSigned {
        let x = self.x as isize - rhs.x as isize;
        let y = self.y as isize - rhs.y as isize;
        return PsSigned { x: x, y: y };
    }
}

impl std::ops::Sub for Ps {
    type Output = PsSigned;

    fn sub(self, rhs: Self) -> PsSigned {
        let x = self.x as isize - rhs.x as isize;
        let y = self.y as isize - rhs.y as isize;
        return PsSigned { x: x, y: y };
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PsSigned {
    pub x: isize,
    pub y: isize,
}

impl PsSigned {
    pub fn manhattan_distance(&self, other: &PsSigned) -> i32 {
        let diffx = self.x.abs_diff(other.x);
        let diffy = self.y.abs_diff(other.y);
        return (diffx + diffy).to_i32().unwrap();
    }
}

impl From<(isize, isize)> for PsSigned {
    fn from(value: (isize, isize)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

pub trait XYprovider {
    fn get_xy(&self) -> (isize, isize);
}

impl XYprovider for Ps {
    fn get_xy(&self) -> (isize, isize) {
        return (self.x as isize, self.y as isize);
    }
}

impl XYprovider for PsSigned {
    fn get_xy(&self) -> (isize, isize) {
        return (self.x, self.y);
    }
}

pub trait PsProvider {
    fn get_current_ps(&self) -> Ps;
}

impl PsProvider for Ps {
    fn get_current_ps(&self) -> Ps {
        return self.clone();
    }
}

pub trait PsSignedProvider {
    fn get_current_ps(&self) -> PsSigned;
}

impl Into<Ps> for PsSigned {
    fn into(self) -> Ps {
        Ps {
            x: self.x as usize,
            y: self.y as usize,
        }
    }
}

