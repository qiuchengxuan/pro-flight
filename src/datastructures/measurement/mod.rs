use core::{fmt, ops};

use fixed_point::{fixed_point, FixedPoint};
use integer_sqrt::IntegerSquareRoot;
use nalgebra::Vector3;
use serde::ser::SerializeStruct;

pub mod axes;
pub mod displacement;
pub mod distance;
pub mod euler;
pub mod rotation;
pub mod unit;
#[macro_use]
pub mod voltage;

use distance::Distance;
use unit::CentiMeter;

pub use axes::Axes;
pub use rotation::Rotation;

pub type Velocity<T, U> = distance::Distance<T, U>;
pub type VelocityVector<T, U> = displacement::DistanceVector<T, U>;

impl Into<Vector3<f32>> for VelocityVector<f32, unit::MpS> {
    fn into(self) -> Vector3<f32> {
        Vector3::new(self.x.value(), self.y.value(), self.z.value())
    }
}

impl From<Vector3<f32>> for VelocityVector<f32, unit::MpS> {
    fn from(vector: Vector3<f32>) -> Self {
        Self::new(vector[0], vector[1], vector[2], unit::MpS)
    }
}

pub type Temperature = i16;
pub type Altitude = Distance<i32, CentiMeter>;

pub type Heading = FixedPoint<i32, 1>;
pub type Course = FixedPoint<i32, 1>;

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Bias {
    pub x: FixedPoint<i32, 5>,
    pub y: FixedPoint<i32, 5>,
    pub z: FixedPoint<i32, 5>,
}

macro_rules! bias {
    ($x:literal, $y:literal, $z:literal) => {
        Bias { x: fixed_point!($x, 5i32), y: fixed_point!($y, 5i32), z: fixed_point!($z, 5i32) }
    };
}

impl Default for Bias {
    fn default() -> Self {
        bias!(0.0, 0.0, 0.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Gain {
    pub x: FixedPoint<u16, 4>,
    pub y: FixedPoint<u16, 4>,
    pub z: FixedPoint<u16, 4>,
}

macro_rules! gain {
    ($x:literal, $y:literal, $z:literal) => {
        Gain { x: fixed_point!($x, 4u16), y: fixed_point!($y, 4u16), z: fixed_point!($z, 4u16) }
    };
}

impl Default for Gain {
    fn default() -> Self {
        gain!(1.0, 1.0, 1.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Measurement {
    pub axes: Axes,
    pub sensitive: u16,
}

impl Measurement {
    pub fn zero(self, bias: &Bias) -> Self {
        Self {
            axes: Axes {
                x: self.axes.x + bias.x.0 * self.sensitive as i32 / bias.x.exp() as i32,
                y: self.axes.y + bias.y.0 * self.sensitive as i32 / bias.y.exp() as i32,
                z: self.axes.z + bias.z.0 * self.sensitive as i32 / bias.z.exp() as i32,
            },
            sensitive: self.sensitive,
        }
    }

    pub fn gain(self, gain: &Gain) -> Self {
        Self {
            axes: Axes {
                x: self.axes.x * gain.x.0 as i32 / gain.x.exp() as i32,
                y: self.axes.y * gain.y.0 as i32 / gain.y.exp() as i32,
                z: self.axes.z * gain.z.0 as i32 / gain.z.exp() as i32,
            },
            sensitive: self.sensitive,
        }
    }

    pub fn rotate(self, rotation: Rotation) -> Self {
        Self { axes: self.axes.rotate(rotation), sensitive: self.sensitive }
    }
}

impl PartialOrd for Measurement {
    fn partial_cmp(self: &Self, other: &Self) -> Option<core::cmp::Ordering> {
        self.axes.partial_cmp(&other.axes)
    }
}

impl ops::Add for Measurement {
    type Output = Measurement;

    fn add(self, other: Self) -> Self::Output {
        Self::Output { axes: self.axes + other.axes, sensitive: other.sensitive }
    }
}

impl ops::Sub for Measurement {
    type Output = Measurement;

    fn sub(self, other: Self) -> Self::Output {
        Self::Output { axes: self.axes - other.axes, sensitive: other.sensitive }
    }
}

impl ops::Div<i32> for Measurement {
    type Output = Measurement;

    fn div(self, div: i32) -> Self::Output {
        Self::Output { axes: self.axes / div, sensitive: self.sensitive }
    }
}

impl Into<(f32, f32, f32)> for Measurement {
    fn into(self) -> (f32, f32, f32) {
        (
            self.axes.x as f32 / self.sensitive as f32,
            self.axes.y as f32 / self.sensitive as f32,
            self.axes.z as f32 / self.sensitive as f32,
        )
    }
}

impl Into<Vector3<f32>> for Measurement {
    fn into(self) -> Vector3<f32> {
        let (x, y, z) = self.into();
        Vector3::new(x, y, z)
    }
}

impl Default for Measurement {
    fn default() -> Self {
        Self { axes: Default::default(), sensitive: u16::MAX }
    }
}

impl serde::Serialize for Measurement {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let (x, y, z) = (self.axes.x as f32, self.axes.y as f32, self.axes.z as f32);
        let sensitive = self.sensitive as f32;
        let mut struct_ = serializer.serialize_struct("Measurement", 3)?;
        struct_.serialize_field("x", &(x / sensitive))?;
        struct_.serialize_field("y", &(y / sensitive))?;
        struct_.serialize_field("z", &(z / sensitive))?;
        struct_.end()
    }
}

impl<'d> serde::Deserialize<'d> for Measurement {
    fn deserialize<D: serde::Deserializer<'d>>(deserializer: D) -> Result<Self, D::Error> {
        struct MeasurementVisitor;

        impl<'de> serde::de::Visitor<'de> for MeasurementVisitor {
            type Value = Measurement;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Measurement")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut measurement = Measurement { sensitive: u16::MAX, ..Default::default() };
                while let Some((key, value)) = access.next_entry::<&str, f32>()? {
                    let value = (value * u16::MAX as f32) as i32;
                    match key {
                        "x" => measurement.axes.x = value,
                        "y" => measurement.axes.y = value,
                        "z" => measurement.axes.z = value,
                        _ => continue,
                    }
                }
                Ok(measurement)
            }
        }

        deserializer.deserialize_map(MeasurementVisitor {})
    }
}

pub const GRAVITY: f32 = 9.80665;

#[derive(Debug, Default, Copy, Clone)]
pub struct Acceleration(pub Measurement);

impl serde::Serialize for Acceleration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'d> serde::Deserialize<'d> for Acceleration {
    fn deserialize<D: serde::Deserializer<'d>>(deserializer: D) -> Result<Self, D::Error> {
        Measurement::deserialize(deserializer).map(|v| Self(v))
    }
}

impl Acceleration {
    pub fn g_force(&self) -> u8 {
        let axes = self.0.axes;
        let (x, y, z) = (axes.x, axes.y, axes.z);
        let square_sum = x * x + y * y + z * z;
        if square_sum > 0 {
            let g_force = square_sum.integer_sqrt();
            (g_force * 10 / self.0.sensitive as i32) as u8
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
        Distance::new(((1013_25 - self.0 as isize) * 82 / 10) as i32, CentiMeter)
    }
}

pub type Magnetism = Measurement;

mod test {
    #[test]
    fn test_distance_unit_convert() {
        use super::{Altitude, Distance};
        use crate::datastructures::measurement::unit::{CentiMeter, Feet, Meter, NauticalMile};

        let altitude = Altitude::new(1000, CentiMeter);
        assert_eq!(altitude.to_unit(Meter), Distance::new(10, Meter));
        assert_eq!(altitude.to_unit(Feet), Distance::new(33, Feet));

        let distance = Distance::new(1, NauticalMile);
        assert_eq!(distance.to_unit(Meter), Distance::new(1852, Meter));
    }

    #[test]
    fn test_speed_unit_convert() {
        use super::Velocity;
        use crate::datastructures::measurement::unit::{FTpM, KMpH, Knot, Meter};

        let knot = Velocity::new(186, KMpH);
        assert_eq!(knot.to_unit(Knot), Velocity::new(100, Knot));

        let meter = Velocity::new(1800, FTpM);
        assert_eq!(meter.to_unit(Meter), Velocity::new(9, Meter));
    }

    #[test]
    fn test_gain() {
        use super::{Axes, Gain, Measurement};
        use fixed_point::{fixed_point, FixedPoint};

        let measurement = Measurement { axes: Axes { x: 100, y: 200, z: 300 }, sensitive: 0 };
        let measurement = measurement.gain(&gain!(1.01, 1.02, 1.03));
        assert_eq!(measurement.axes, Axes { x: 101, y: 204, z: 309 });
    }
}
