use crate::datastructures::measurement::{unit, Altitude, Velocity};

pub struct Variometer {
    prev: Altitude,
    interval: usize, // unit ms
}

impl Variometer {
    pub fn new(interval: usize) -> Self {
        Self { prev: Default::default(), interval }
    }

    pub fn update(&mut self, d: Altitude) -> Velocity<i32, unit::CMpS> {
        let v = (d - self.prev) * self.interval as i32 / 1000;
        self.prev = d;
        v.to_unit(unit::CMpS)
    }
}
