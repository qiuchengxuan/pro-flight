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
