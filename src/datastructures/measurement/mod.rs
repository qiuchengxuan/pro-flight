use core::ops::{Add, Sub};

use integer_sqrt::IntegerSquareRoot;
#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::Vector3;

pub mod battery;
pub mod euler;

pub use euler::DEGREE_PER_DAG;

#[derive(Copy, Clone, PartialEq)]
pub enum DistanceUnit {
    CentiMeter = 1,
    Feet = 330,
    Meter = 100,
    KiloMeter = 100_000,
    NauticalMile = 1852 * 100,
}

#[derive(Default, Copy, Clone, PartialEq)]
pub struct Distance<T: Default + Copy + Clone + PartialEq>(pub T);

impl<T: Add<Output = T> + Copy + Default + PartialEq> Add for Distance<T> {
    type Output = Distance<T::Output>;
    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl<T: Sub<Output = T> + Copy + Default + PartialEq> Sub for Distance<T> {
    type Output = Distance<T::Output>;
    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl<T: Into<isize> + Copy + Default + PartialEq> sval::value::Value for Distance<T> {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.i64(self.0.into() as i64)
    }
}

impl<T: Into<isize> + Copy + Default + PartialEq> Distance<T> {
    #[inline]
    pub fn convert(self, from: DistanceUnit, to: DistanceUnit, scale: isize) -> isize {
        let raw: isize = self.0.into();
        match (from, to) {
            (DistanceUnit::CentiMeter, _) => raw * scale / to as isize,
            _ => raw * scale * from as isize / to as isize,
        }
    }
}

impl<T: Into<f32> + Copy + Default + PartialEq> Into<f32> for Distance<T> {
    fn into(self) -> f32 {
        self.0.into() / 100.0
    }
}

pub type Temperature = i16;
pub type Altitude = Distance<isize>;
pub type Velocity = i16;

#[derive(Copy, Clone, Debug, PartialEq, Default, Value)]
pub struct Axes {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Axes {
    pub fn average(self: &Self, other: &Self) -> Self {
        Self { x: (self.x + other.x) / 2, y: (self.y + other.y) / 2, z: (self.z + other.z) / 2 }
    }

    pub fn calibrated(&self, calibration: &Self) -> Self {
        Self { x: self.x - calibration.x, y: self.y - calibration.y, z: self.z - calibration.z }
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

#[derive(Copy, Clone, PartialEq, Value)]
pub struct Measurement {
    pub axis: Axes,
    pub sensitive: i32,
}

impl Measurement {
    pub fn calibrated(self, axis: &Axes) -> Self {
        Self { axis: self.axis.calibrated(axis), sensitive: self.sensitive }
    }
}

impl PartialOrd for Measurement {
    fn partial_cmp(self: &Self, other: &Self) -> Option<core::cmp::Ordering> {
        self.axis.partial_cmp(&other.axis)
    }
}

impl Into<Vector3<f32>> for Measurement {
    fn into(self) -> Vector3<f32> {
        Vector3::new(
            self.axis.x as f32 / self.sensitive as f32,
            self.axis.y as f32 / self.sensitive as f32,
            self.axis.z as f32 / self.sensitive as f32,
        )
    }
}

impl Default for Measurement {
    fn default() -> Self {
        Self { axis: Default::default(), sensitive: 1 }
    }
}

#[derive(Default, Copy, Clone)]
pub struct Acceleration(pub Measurement);

impl Acceleration {
    pub fn calibrated(self, axis: &Axes) -> Self {
        return Self(self.0.calibrated(axis));
    }
}

impl Acceleration {
    pub fn g_force(&self) -> u8 {
        let axis = self.0.axis;
        let (x, y, z) = (axis.x, axis.y, axis.z);
        let square_sum = x * x + y * y + z * z;
        if square_sum > 0 {
            let g_force = square_sum.integer_sqrt();
            (g_force * 10 / self.0.sensitive) as u8
        } else {
            0
        }
    }
}

pub type Gyro = Measurement;

#[derive(Copy, Clone, Default)]
pub struct Pressure(pub u32); // unit of Pa

impl Into<Altitude> for Pressure {
    fn into(self) -> Altitude {
        Distance(((1013_25 - self.0 as isize) * 82 / 10) as isize)
    }
}
