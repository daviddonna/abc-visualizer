extern crate rand;

use std::ops::{Add, Sub, Mul, Div};
use std::cmp::{min, max};
use self::rand::{thread_rng, Rng};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coords {
    pub x: i32,
    pub y: i32,
}

impl Coords {
    pub fn random(min: &Coords, max: &Coords) -> Coords {
        Coords {
            x: Coords::random_coord(min.x, max.x),
            y: Coords::random_coord(min.y, max.y),
        }
    }

    pub fn magnitude(&self) -> i32 {
        ((self.x.pow(2) + self.y.pow(2)) as f64).sqrt().round() as i32
    }

    fn random_coord(min: i32, max: i32) -> i32 {
        thread_rng().gen_range(min, max + 1)
    }

    pub fn clamp(self, min_c: &Coords, max_c: &Coords) -> Coords {
        Coords {
            x: min(max_c.x, max(min_c.x, self.x)),
            y: min(max_c.y, max(min_c.y, self.y)),
        }
    }
}

impl Add for Coords {
    type Output = Coords;
    fn add(self, other: Coords) -> Coords {
        Coords {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Coords {
    type Output = Coords;
    fn sub(self, other: Coords) -> Coords {
        Coords {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<i32> for Coords {
    type Output = Coords;
    fn mul(self, coefficient: i32) -> Coords {
        Coords {
            x: self.x * coefficient,
            y: self.y * coefficient,
        }
    }
}

impl Div<i32> for Coords {
    type Output = Coords;
    fn div(self, coefficient: i32) -> Coords {
        Coords {
            x: self.x / coefficient,
            y: self.y / coefficient,
        }
    }
}
