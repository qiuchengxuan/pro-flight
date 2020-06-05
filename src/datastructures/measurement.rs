#[allow(unused)] // false warning
use micromath::F32Ext;
use nalgebra::Quaternion;

pub const DEGREE_PER_DAG: f32 = 180.0 / core::f32::consts::PI;

#[derive(Default, Copy, Clone)]
pub struct Euler {
    pub phi: f32,   // roll
    pub theta: f32, // pitch
    pub psi: f32,   // yaw
}

impl core::fmt::Display for Euler {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{{\"phi\":{},\"theta\":{},\"psi\":{}}}", self.phi, self.theta, self.psi)
    }
}

pub fn quaternion_to_euler(q: Quaternion<f32>) -> Euler {
    let (i, j, k, w) = (q[0], q[1], q[2], q[3]);
    let phi = (2.0 * (w * i + j * k)).atan2(1.0 - 2.0 * (i * i + j * j)) * DEGREE_PER_DAG;
    let theta = (2.0 * (w * j - k * j)).asin() * DEGREE_PER_DAG;
    let psi = (2.0 * (w * k + i * j)).atan2(1.0 - 2.0 * (j * j + k * k)) * DEGREE_PER_DAG;
    Euler { phi, theta, psi }
}

pub struct Altitude(pub i32); // in units of centimeter

impl Altitude {
    pub fn as_feet(self) -> i32 {
        self.0 / 33
    }

    pub fn as_meter(self) -> i32 {
        self.0 / 10
    }
}
