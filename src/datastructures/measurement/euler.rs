#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::{Quaternion, UnitQuaternion};

pub const DEGREE_PER_DAG: f32 = 180.0 / core::f32::consts::PI;

#[derive(Default, Copy, Clone, Value, Debug)]
pub struct Euler {
    pub phi: f32,   // around x axis
    pub theta: f32, // around y axis
    pub psi: f32,   // around z axis
}

impl core::ops::Mul<f32> for Euler {
    type Output = Self;
    fn mul(self, m: f32) -> Euler {
        Euler { phi: self.phi * m, theta: self.theta * m, psi: self.psi * m }
    }
}

impl core::ops::Div<f32> for Euler {
    type Output = Self;
    fn div(self, d: f32) -> Euler {
        Euler { phi: self.phi / d, theta: self.theta / d, psi: self.psi / d }
    }
}

impl Into<(isize, isize, isize)> for Euler {
    fn into(self) -> (isize, isize, isize) {
        (self.phi as isize, self.theta as isize, self.psi as isize)
    }
}

macro_rules! pow2 {
    ($x:expr) => {
        $x * $x
    };
}

impl From<UnitQuaternion<f32>> for Euler {
    fn from(q: UnitQuaternion<f32>) -> Self {
        let (i, j, k, w) = (q[0], q[1], q[2], q[3]);
        let phi = (2.0 * (w * i + j * k)).atan2(1.0 - 2.0 * (pow2!(i) + pow2!(j)));
        let theta = (2.0 * (w * j - i * k)).asin();
        let psi = (2.0 * (w * k + i * j)).atan2(1.0 - 2.0 * (pow2!(j) + pow2!(k)));
        Self { phi, theta, psi }
    }
}

impl Into<UnitQuaternion<f32>> for Euler {
    fn into(self) -> UnitQuaternion<f32> {
        let euler = self / 2.0;
        let cos_phi = euler.phi.cos();
        let sin_phi = euler.phi.sin();
        let cos_theta = euler.theta.cos();
        let sin_theta = euler.theta.sin();
        let cos_psi = euler.psi.cos();
        let sin_psi = euler.psi.sin();
        let w = cos_theta * cos_phi * cos_psi + sin_theta * sin_phi * sin_psi;
        let i = cos_theta * sin_phi * cos_psi + sin_theta * cos_phi * sin_psi;
        let j = sin_theta * cos_phi * cos_psi - cos_theta * sin_phi * sin_psi;
        let k = cos_theta * cos_phi * sin_psi - sin_theta * sin_phi * cos_psi;
        UnitQuaternion::new_normalize(Quaternion::new(w, i, j, k))
    }
}

mod test {
    #[test]
    fn test_euler() {
        use nalgebra::UnitQuaternion;

        use super::{Euler, DEGREE_PER_DAG};

        let euler = Euler { phi: 0.0, theta: 45.0, psi: 0.0 } / DEGREE_PER_DAG;
        let quaternion: UnitQuaternion<f32> = euler.into();
        let revert: Euler = quaternion.into();
        let euler_isize: (isize, isize, isize) = euler.into();
        let revert_isize: (isize, isize, isize) = revert.into();
        assert_eq!(euler_isize, revert_isize);
    }
}
