use core::ops;

use fixed_point::FixedPoint;

pub mod euler;
pub mod unit;
#[macro_use]
pub mod voltage;
pub mod vector;
pub mod vector3;

use unit::CentiMeter;

pub use vector3::{Coordinate, Frame, ENU, X, Y, Z};

pub type Attitude = euler::Euler;
pub type Distance<T, U> = vector::Vector<T, U>;
pub type Velocity<T, U> = vector::Vector<T, U>;

pub type Displacement<T, U, C> = vector3::Vector3<T, U, C>;
pub type VelocityVector<T, U, C> = vector3::Vector3<T, U, C>;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Temperature(i16);

#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Altitude(pub Distance<i32, CentiMeter>);

impl Altitude {
    pub fn is_zero(&self) -> bool {
        self.0.raw == 0
    }
}

impl ops::Sub<Self> for Altitude {
    type Output = Distance<i32, CentiMeter>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

impl ops::Add<Distance<i32, CentiMeter>> for Altitude {
    type Output = Self;

    fn add(self, rhs: Distance<i32, CentiMeter>) -> Self {
        Self(self.0 + rhs)
    }
}

impl ops::AddAssign<Distance<i32, CentiMeter>> for Altitude {
    fn add_assign(&mut self, rhs: Distance<i32, CentiMeter>) {
        self.0 += rhs;
    }
}

impl ops::Sub<Distance<i32, CentiMeter>> for Altitude {
    type Output = Self;

    fn sub(self, rhs: Distance<i32, CentiMeter>) -> Self {
        Self(self.0 - rhs)
    }
}

impl From<Altitude> for Displacement<i32, CentiMeter, ENU> {
    fn from(altitude: Altitude) -> Self {
        Self::new(0, 0, altitude.0.raw, CentiMeter, ENU)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Heading(pub FixedPoint<i32, 1>);

#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Course(pub FixedPoint<i32, 1>);

pub const GRAVITY: f32 = 9.80665;

#[derive(Copy, Clone, Debug, Default)]
pub struct Acceleration<C: Copy + Default>(pub vector3::Vector3<f32, unit::M2s, C>);

impl<C: Copy + Default + Coordinate> Acceleration<C> {
    pub fn new(value: nalgebra::Vector3<f32>, coordinate: C) -> Self {
        Self(vector3::Vector3::from(value, unit::M2s, coordinate))
    }
}

impl<C: Copy + Default> Acceleration<C> {
    pub fn g_force(&self) -> u8 {
        (self.0.scalar().raw / GRAVITY * 10.0) as u8
    }
}

impl<'a, C: Copy + Default> serde::Deserialize<'a> for Acceleration<C> {
    fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let v = vector3::Vector3::deserialize(deserializer)?;
        Ok(Self(v))
    }
}

#[derive(Copy, Clone, Debug, Default, Serialize)]
pub struct Gyro<U: Copy + Default>(pub vector3::Vector3<f32, U, Frame>);

impl<U: Copy + Default> Gyro<U> {
    pub fn new(value: nalgebra::Vector3<f32>, u: U) -> Self {
        Self(vector3::Vector3::from(value, u, Frame))
    }
}

impl<'a, U: Copy + Default> serde::Deserialize<'a> for Gyro<U> {
    fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let v = vector3::Vector3::deserialize(deserializer)?;
        Ok(Self(v))
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Pressure(pub u32); // unit of Pa

impl Into<Altitude> for Pressure {
    fn into(self) -> Altitude {
        Altitude(Distance::new(((1013_25 - self.0 as isize) * 82 / 10) as i32, CentiMeter))
    }
}

mod test {
    #[test]
    fn test_distance_unit_convert() {
        use super::{
            unit::{CentiMeter, Feet, Meter, NauticalMile},
            Distance,
        };

        let altitude = Distance::new(1000, CentiMeter);
        assert_eq!(altitude.u(Meter), Distance::new(10, Meter));
        assert_eq!(altitude.u(Feet), Distance::new(33, Feet));

        let distance = Distance::new(1, NauticalMile);
        assert_eq!(distance.u(Meter), Distance::new(1852, Meter));
    }

    #[test]
    fn test_velocity_unit_convert() {
        use super::{
            unit::{FTmin, KMh, Knot, Meter},
            Velocity,
        };

        let knot = Velocity::new(186, KMh);
        assert_eq!(knot.u(Knot), Velocity::new(100, Knot));

        let meter = Velocity::new(1800, FTmin);
        assert_eq!(meter.u(Meter), Velocity::new(9, Meter));
    }
}
