use crate::{
    algorithm::lpf::LPF,
    types::measurement::{unit, Altitude, Velocity},
};

pub struct Variometer {
    lpf: LPF<i16>,
    prev: Altitude,
    interval: usize, // unit ms
}

impl Variometer {
    pub fn new(interval: usize) -> Self {
        Self {
            lpf: LPF::<i16>::new(1.0 / interval as f32, 1.0),
            prev: Default::default(),
            interval,
        }
    }

    pub fn update(&mut self, d: Altitude) -> Velocity<i32, unit::CMs> {
        let v = (d - self.prev) * self.interval as i32 / 1000;
        self.prev = d;
        Velocity::new(self.lpf.filter(v.raw as i16) as i32, unit::CMs)
    }
}
