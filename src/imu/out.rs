use nalgebra::UnitQuaternion;
use serde::ser::SerializeStruct;

use crate::types::measurement::{unit::DEGs, Acceleration, Attitude, Gyro, ENU};

#[derive(Copy, Clone, Debug, Default)]
pub struct IMU {
    pub acceleration: Acceleration<ENU>,
    pub attitude: Attitude,
    pub gyro: Gyro<DEGs>,
    pub quaternion: UnitQuaternion<f32>,
}

impl serde::Serialize for IMU {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut struct_ = serializer.serialize_struct("IMU", 4)?;
        let acceleration: [f32; 3] = self.acceleration.0.into();
        struct_.serialize_field("acceleration", &acceleration[..])?;
        struct_.serialize_field("attitude", &self.attitude)?;
        let gyro: [f32; 3] = self.gyro.0.into();
        struct_.serialize_field("gyro", &gyro[..])?;
        let q = &self.quaternion;
        let value: [f32; 4] = [q.i, q.j, q.k, q.w];
        struct_.serialize_field("quaternion", &value[..])?;
        struct_.end()
    }
}

mod test {
    #[test]
    fn test_serialize() {
        use serde_json::json;

        use super::IMU;

        let expected = json!({
            "acceleration": [0.0, 0.0, 0.0],
            "attitude": {"roll": 0.0, "pitch": 0.0, "yaw": 0.0},
            "gyro": [0.0, 0.0, 0.0],
            "quaternion": [0.0, 0.0, 0.0, 1.0]
        });
        let imu = IMU::default();
        assert_eq!(expected, serde_json::to_value(&imu).unwrap());
    }
}
