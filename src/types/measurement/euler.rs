use core::f32::consts::PI;

#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::UnitQuaternion;

pub const DEGREE_PER_DAG: f32 = 180.0 / PI;

#[derive(Default, Copy, Clone, Serialize, Debug, PartialEq)]
pub struct Euler {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl Euler {
    pub fn new(roll: f32, pitch: f32, yaw: f32) -> Self {
        Self { roll, pitch, yaw }
    }
}

impl core::ops::Mul<f32> for Euler {
    type Output = Self;

    fn mul(self, m: f32) -> Euler {
        Euler { roll: self.roll * m, pitch: self.pitch * m, yaw: self.yaw * m }
    }
}

impl core::ops::Div<f32> for Euler {
    type Output = Self;

    fn div(self, d: f32) -> Euler {
        Euler { roll: self.roll / d, pitch: self.pitch / d, yaw: self.yaw / d }
    }
}

impl Into<(isize, isize, isize)> for Euler {
    fn into(self) -> (isize, isize, isize) {
        (self.roll as isize, self.pitch as isize, self.yaw as isize)
    }
}

impl From<UnitQuaternion<f32>> for Euler {
    fn from(q: UnitQuaternion<f32>) -> Self {
        let r01 = 2.0 * (q.i * q.j - q.w * q.k);
        let r11 = q.w * q.w - q.i * q.i + q.j * q.j - q.k * q.k;
        let r20 = 2.0 * (q.i * q.k - q.w * q.j);
        let r21 = 2.0 * (q.w * q.i + q.j * q.k);
        let r22 = q.w * q.w - q.i * q.i - q.j * q.j + q.k * q.k;
        let mut yaw = -r01.atan2(r11);
        if yaw < 0.0 {
            yaw += 2.0 * PI;
        }
        let pitch = if r21.abs() >= 1.0 { (PI / 2.0).copysign(r21) } else { (r21).asin() };
        let roll = -r20.atan2(r22);
        Self { roll, pitch, yaw }
    }
}

mod test {
    #[test]
    fn test_quaternion_to_euler() {
        use nalgebra::{Quaternion, UnitQuaternion};

        use super::{Euler, DEGREE_PER_DAG};

        // roll 90
        let unit = UnitQuaternion::new_normalize(Quaternion::new(0.7071068, 0.0, 0.7071068, 0.0));
        assert_eq!(Euler::new(90.0, 0.0, 0.0), Euler::from(unit) * DEGREE_PER_DAG);

        // pitch 90
        let unit = UnitQuaternion::new_normalize(Quaternion::new(0.7071068, 0.7071068, 0.0, 0.0));
        assert_eq!(Euler::new(-0.0, 90.0, -0.0), Euler::from(unit) * DEGREE_PER_DAG);

        // yaw 0
        let unit = UnitQuaternion::new_normalize(Quaternion::new(1.0, 0.0, 0.0, 0.0));
        assert_eq!(Euler::new(0.0, 0.0, 0.0), Euler::from(unit) * DEGREE_PER_DAG);

        // yaw 90
        let unit = UnitQuaternion::new_normalize(Quaternion::new(0.7071068, 0.0, 0.0, 0.7071068));
        assert_eq!(Euler::new(-0.0, 0.0, 90.0), Euler::from(unit) * DEGREE_PER_DAG);

        // yaw 180
        let unit = UnitQuaternion::new_normalize(Quaternion::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(Euler::new(0.0, 0.0, 180.0), Euler::from(unit) * DEGREE_PER_DAG);

        // yaw 270
        let unit = UnitQuaternion::new_normalize(Quaternion::new(0.7071068, 0.0, 0.0, -0.7071068));
        assert_eq!(Euler::new(0.0, 0.0, 270.0), Euler::from(unit) * DEGREE_PER_DAG);
    }
}
