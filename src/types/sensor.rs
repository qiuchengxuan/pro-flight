use core::str::FromStr;

use fixed_point::{fixed, FixedPoint};
use nalgebra::Vector3;

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Axes {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Bias {
    pub x: FixedPoint<i32, 5>,
    pub y: FixedPoint<i32, 5>,
    pub z: FixedPoint<i32, 5>,
}

impl Into<Vector3<f32>> for Bias {
    fn into(self) -> Vector3<f32> {
        Vector3::new(self.x.into(), self.y.into(), self.z.into())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
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

impl Into<Vector3<f32>> for Gain {
    fn into(self) -> Vector3<f32> {
        Vector3::new(self.x.into(), self.y.into(), self.z.into())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum Rotation {
    NoRotation,
    Degree90,
    Degree180,
    Degree270,
}

impl Default for Rotation {
    fn default() -> Self {
        Self::NoRotation
    }
}

impl Rotation {
    pub fn rotate(&self, v: [f32; 3]) -> [f32; 3] {
        match self {
            Self::NoRotation => v,
            Self::Degree90 => [v[1], v[0], v[2]],
            Self::Degree180 => [-v[0], -v[1], v[2]],
            Self::Degree270 => [-v[1], v[0], v[2]],
        }
    }
}

impl FromStr for Rotation {
    type Err = ();

    fn from_str(name: &str) -> Result<Rotation, ()> {
        match name {
            "0" => Ok(Rotation::NoRotation),
            "90" => Ok(Rotation::Degree90),
            "180" => Ok(Rotation::Degree180),
            "270" => Ok(Rotation::Degree270),
            _ => Err(()),
        }
    }
}

impl serde::Serialize for Rotation {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = match self {
            Self::NoRotation => "0",
            Self::Degree90 => "90",
            Self::Degree180 => "180",
            Self::Degree270 => "270",
        };
        serializer.serialize_str(s)
    }
}
