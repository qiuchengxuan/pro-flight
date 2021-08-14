use serde::ser::SerializeMap;

use crate::datastructures::measurement::{euler::Euler, Altitude};

#[derive(Debug, Default, Copy, Clone)]
pub struct Attitude {
    pub roll: i16,
    pub pitch: i16,
}

impl From<Euler> for Attitude {
    fn from(euler: Euler) -> Self {
        let roll = (-euler.roll * 10.0) as i16;
        let pitch = (-euler.pitch * 10.0) as i16;
        Self { roll, pitch }
    }
}

impl serde::Serialize for Attitude {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("roll", &(self.roll / 10))?;
        map.serialize_entry("pitch", &(self.pitch / 10))?;
        map.end()
    }
}

#[derive(Copy, Clone, Default, Debug, Serialize)]
pub struct Aviation {
    pub altitude: Altitude,
    pub attitude: Attitude,
    pub heading: u16,
    pub height: Altitude,
    pub g_force: u8,
    pub airspeed: u16,
    pub vario: i16,
}

mod test {
    #[test]
    fn test_serialize_aviation() {
        use serde_json::json;

        use super::Aviation;

        let expected = json!({
            "altitude": 0,
            "attitude": {
                "roll": 0,
                "pitch": 0,
            },
            "heading": 0,
            "height": 0,
            "g_force": 0,
            "airspeed": 0,
            "vario": 0,
        });
        let aviation = Aviation::default();
        assert_eq!(expected, serde_json::to_value(&aviation).unwrap());
    }
}
