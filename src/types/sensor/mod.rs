mod axes;
mod rotation;

pub use axes::Axes;
pub use rotation::Rotation;

use core::{fmt, ops};

use fixed_point::{fixed, FixedPoint};
use nalgebra::Vector3;
use serde::ser::SerializeSeq;

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Bias {
    pub x: FixedPoint<i32, 5>,
    pub y: FixedPoint<i32, 5>,
    pub z: FixedPoint<i32, 5>,
}

macro_rules! bias {
    ($x:literal, $y:literal, $z:literal) => {
        Bias { x: fixed!($x), y: fixed!($y), z: fixed!($z) }
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
        Gain { x: fixed!($x), y: fixed!($y), z: fixed!($z) }
    };
}

impl Default for Gain {
    fn default() -> Self {
        gain!(1.0, 1.0, 1.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Readout {
    pub axes: Axes,
    pub sensitive: u16,
}

impl Readout {
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

impl PartialOrd for Readout {
    fn partial_cmp(self: &Self, other: &Self) -> Option<core::cmp::Ordering> {
        self.axes.partial_cmp(&other.axes)
    }
}

impl ops::Add for Readout {
    type Output = Readout;

    fn add(self, other: Self) -> Self::Output {
        Self::Output { axes: self.axes + other.axes, sensitive: other.sensitive }
    }
}

impl ops::Sub for Readout {
    type Output = Readout;

    fn sub(self, other: Self) -> Self::Output {
        Self::Output { axes: self.axes - other.axes, sensitive: other.sensitive }
    }
}

impl ops::Div<i32> for Readout {
    type Output = Readout;

    fn div(self, div: i32) -> Self::Output {
        Self::Output { axes: self.axes / div, sensitive: self.sensitive }
    }
}

impl Into<[f32; 3]> for Readout {
    fn into(self) -> [f32; 3] {
        [
            self.axes.x as f32 / self.sensitive as f32,
            self.axes.y as f32 / self.sensitive as f32,
            self.axes.z as f32 / self.sensitive as f32,
        ]
    }
}

impl Into<Vector3<f32>> for Readout {
    fn into(self) -> Vector3<f32> {
        let array: [f32; 3] = self.into();
        array.into()
    }
}

impl Default for Readout {
    fn default() -> Self {
        Self { axes: Default::default(), sensitive: u16::MAX }
    }
}

impl serde::Serialize for Readout {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let array: [f32; 3] = (*self).into();
        let mut seq = serializer.serialize_seq(Some(3))?;
        for v in array {
            seq.serialize_element(&v)?;
        }
        seq.end()
    }
}

impl<'d> serde::Deserialize<'d> for Readout {
    fn deserialize<D: serde::Deserializer<'d>>(deserializer: D) -> Result<Self, D::Error> {
        struct ReadoutVisitor;

        impl<'de> serde::de::Visitor<'de> for ReadoutVisitor {
            type Value = Readout;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Readout")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut readout = Readout { sensitive: u16::MAX, ..Default::default() };
                while let Some((key, value)) = access.next_entry::<&str, f32>()? {
                    let value = (value * u16::MAX as f32) as i32;
                    match key {
                        "x" => readout.axes.x = value,
                        "y" => readout.axes.y = value,
                        "z" => readout.axes.z = value,
                        _ => continue,
                    }
                }
                Ok(readout)
            }
        }

        deserializer.deserialize_map(ReadoutVisitor {})
    }
}

pub type Magnetism = Readout;

mod test {

    #[test]
    fn test_gain() {
        use super::{Axes, Gain, Readout};
        use fixed_point::{fixed, FixedPoint};

        let readout = Readout { axes: Axes { x: 100, y: 200, z: 300 }, sensitive: 0 };
        let readout = readout.gain(&gain!(1.01, 1.02, 1.03));
        assert_eq!(readout.axes, Axes { x: 101, y: 204, z: 309 });
    }
}
