use crate::datastructures::measurement::{Altitude, Distance};

const MILLI: isize = 1000;

#[derive(Default, Copy, Clone)]
pub struct LatLongMilliSecond(isize);

impl LatLongMilliSecond {
    pub fn new(degree: i16, minute: u8, second: u8) -> Self {
        Self((degree as isize * 3600 + minute as isize * 60 + second as isize) * MILLI)
    }
}

impl core::ops::Sub for LatLongMilliSecond {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

pub type Longitude = LatLongMilliSecond;
pub type Latitude = LatLongMilliSecond;

#[derive(Default, Copy, Clone)]
pub struct Displacement(Distance, Distance, Distance);

impl Into<(f32, f32, f32)> for Displacement {
    fn into(self) -> (f32, f32, f32) {
        (self.0.into(), self.1.into(), self.2.into())
    }
}

#[derive(Default, Copy, Clone)]
pub struct Position(Longitude, Latitude, Altitude);

impl core::ops::Sub for Position {
    type Output = Displacement;

    fn sub(self, other: Self) -> Displacement {
        let delta = (self.0 - other.0).0;
        let second = delta / 1000;
        let sub_second = delta % 1000;
        let x = second * 30_715 / 10 + sub_second * 30_715 / MILLI / 10;

        let delta = (self.1 - other.1).0;
        let second = delta / 1000;
        let sub_second = delta % 1000;
        let y = second * 30_92 + sub_second * 30_92 / MILLI;
        Displacement(Distance(x), Distance(y), self.2 - other.2)
    }
}
