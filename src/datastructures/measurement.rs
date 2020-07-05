#[allow(unused)] // false warning
use micromath::F32Ext;
use nalgebra::Quaternion;

pub const DEGREE_PER_DAG: f32 = 180.0 / core::f32::consts::PI;

#[derive(Default, Copy, Clone, Value)]
pub struct Euler {
    pub phi: f32,
    pub theta: f32,
    pub psi: f32,
}

macro_rules! pow2 {
    ($x:expr) => {
        $x * $x
    };
}

pub fn quaternion_to_euler(q: &Quaternion<f32>) -> Euler {
    let (i, j, k, w) = (q[0], q[1], q[2], q[3]);
    let phi = (2.0 * (w * i + j * k)).atan2(1.0 - 2.0 * (pow2!(i) + pow2!(j))) * DEGREE_PER_DAG;
    let theta = (2.0 * (w * j - i * k)).asin() * DEGREE_PER_DAG;
    let psi = (2.0 * (w * k + i * j)).atan2(1.0 - 2.0 * (pow2!(j) + pow2!(k))) * DEGREE_PER_DAG;
    Euler { phi, theta, psi }
}

#[derive(Default, Copy, Clone)]
pub struct Distance(pub isize); // in units of centimeter

impl Distance {
    pub fn as_feet(self) -> isize {
        self.0 / 33
    }

    pub fn as_meter(self) -> isize {
        self.0 / 10
    }
}

impl core::ops::Sub for Distance {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl Into<f32> for Distance {
    fn into(self) -> f32 {
        self.0 as f32 / 100.0
    }
}

pub type Altitude = Distance;
