use nalgebra::Vector3;

use crate::datastructures::measurement::Altitude;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Measurement {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub sensitive: f32,
}

impl core::fmt::Display for Measurement {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "{{\"x\":{},\"y\":{},\"z\":{},\"sensitive\":{}}}",
            self.x, self.y, self.z, self.sensitive
        )
    }
}

impl Measurement {
    pub fn average(self: &Self, other: &Self) -> Self {
        Self {
            x: ((self.x as i32 + other.x as i32) / 2) as i16,
            y: ((self.y as i32 + other.y as i32) / 2) as i16,
            z: ((self.z as i32 + other.z as i32) / 2) as i16,
            sensitive: other.sensitive,
        }
    }

    pub fn calibrated(self, calibration: &Self) -> Self {
        Self {
            x: self.x - calibration.x,
            y: self.y - calibration.y,
            z: self.z - calibration.z,
            sensitive: self.sensitive,
        }
    }
}

impl PartialOrd for Measurement {
    fn partial_cmp(self: &Self, other: &Self) -> Option<core::cmp::Ordering> {
        if self.x > other.x || self.y > other.y || self.z > other.z {
            Some(core::cmp::Ordering::Greater)
        } else {
            Some(core::cmp::Ordering::Less)
        }
    }
}

impl Into<(f32, f32, f32)> for Measurement {
    fn into(self) -> (f32, f32, f32) {
        (
            self.x as f32 / self.sensitive,
            self.y as f32 / self.sensitive,
            self.z as f32 / self.sensitive,
        )
    }
}

impl Into<Vector3<f32>> for Measurement {
    fn into(self) -> Vector3<f32> {
        Vector3::new(
            self.x as f32 / self.sensitive,
            self.y as f32 / self.sensitive,
            self.z as f32 / self.sensitive,
        )
    }
}

impl Default for Measurement {
    fn default() -> Self {
        Self { x: 0, y: 0, z: 0, sensitive: 1.0 }
    }
}

pub type Acceleration = Measurement;
pub type Gyro = Measurement;
pub type Temperature<T> = T;

#[derive(Copy, Clone)]
pub struct Pressure(pub u32); // unit of Pa

impl Pressure {
    pub fn to_sea_level_altitude(self) -> Altitude {
        Altitude((1013 - (self.0 / 100) as i32) * 82)
    }
}
