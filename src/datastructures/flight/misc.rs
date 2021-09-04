use nalgebra::UnitQuaternion;
use serde::ser::SerializeMap;

use crate::datastructures::{
    measurement::{displacement::DistanceVector, unit, voltage::Voltage},
    RSSI,
};

#[derive(Copy, Clone, Debug, Default)]
pub struct Misc {
    pub voltage: Voltage,
    pub displacement: DistanceVector<i32, unit::CentiMeter>,
    pub quaternion: UnitQuaternion<f32>,
    pub rssi: RSSI,
}

impl serde::Serialize for Misc {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(5))?;
        map.serialize_entry("displacement", &self.displacement)?;
        let q = &self.quaternion;
        let value: [f32; 4] = [q.i, q.j, q.k, q.w];
        map.serialize_entry("quaternion", &value[..])?;
        map.serialize_entry("rssi", &self.rssi)?;
        map.serialize_entry("voltage", &self.voltage)?;
        map.end()
    }
}

mod test {
    #[test]
    fn test_serialize_misc() {
        use serde_json::json;

        use super::Misc;

        let expected = json!({
            "displacement": {
                "x": 0,
                "y": 0,
                "z": 0,
            },
            "quaternion": [0.0, 0.0, 0.0, 1.0],
            "rssi": 0,
            "voltage": 0.0
        });
        let misc = Misc::default();
        assert_eq!(expected, serde_json::to_value(&misc).unwrap());
    }
}
