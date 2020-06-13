use nalgebra::Vector3;

use crate::datastructures::measurement::Altitude;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Axes {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Axes {
    pub fn average(self: &Self, other: &Self) -> Self {
        Self { x: (self.x + other.x) / 2, y: (self.y + other.y) / 2, z: (self.z + other.z) / 2 }
    }

    pub fn calibrated(&mut self, calibration: &Self) {
        self.x -= calibration.x;
        self.y -= calibration.y;
        self.z -= calibration.z;
    }
}

impl PartialOrd for Axes {
    fn partial_cmp(self: &Self, other: &Self) -> Option<core::cmp::Ordering> {
        if self.x > other.x || self.y > other.y || self.z > other.z {
            Some(core::cmp::Ordering::Greater)
        } else {
            Some(core::cmp::Ordering::Less)
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    pub axes: Axes,
    pub sensitive: i32,
}

impl PartialOrd for Measurement {
    fn partial_cmp(self: &Self, other: &Self) -> Option<core::cmp::Ordering> {
        self.axes.partial_cmp(&other.axes)
    }
}

impl Into<(f32, f32, f32)> for Measurement {
    fn into(self) -> (f32, f32, f32) {
        (
            (self.axes.x as f32 / self.sensitive as f32),
            (self.axes.y as f32 / self.sensitive as f32),
            (self.axes.z as f32 / self.sensitive as f32),
        )
    }
}

impl Into<Vector3<f32>> for Measurement {
    fn into(self) -> Vector3<f32> {
        Vector3::new(
            self.axes.x as f32 / self.sensitive as f32,
            self.axes.y as f32 / self.sensitive as f32,
            self.axes.z as f32 / self.sensitive as f32,
        )
    }
}

impl Default for Measurement {
    fn default() -> Self {
        Self { axes: Default::default(), sensitive: 1 }
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
