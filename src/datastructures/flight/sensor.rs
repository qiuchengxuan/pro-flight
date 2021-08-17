use serde::ser::SerializeMap;

use crate::datastructures::measurement::{Acceleration, Course, Gyro, Magnetism};

#[derive(Copy, Clone, Debug)]
pub struct GNSS {
    pub fixed: bool,
    pub course: Course,
}

impl serde::Serialize for GNSS {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(if self.fixed { 2 } else { 1 }))?;
        map.serialize_entry("fixed", &self.fixed)?;
        if self.fixed {
            map.serialize_entry("course", &self.course)?;
        }
        map.end()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Sensor {
    pub acceleration: Acceleration,
    pub gyro: Gyro,
    pub magnetism: Option<Magnetism>,
    pub gnss: Option<GNSS>,
}

impl Default for Sensor {
    fn default() -> Self {
        Self {
            acceleration: Acceleration::default(),
            gyro: Gyro::default(),
            magnetism: None,
            gnss: None,
        }
    }
}

impl serde::Serialize for Sensor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(4))?;
        map.serialize_entry("acceleration", &self.acceleration)?;
        map.serialize_entry("gyro", &self.gyro)?;
        map.serialize_entry("magnetism", &self.magnetism)?;
        map.serialize_entry("gnss", &self.gnss)?;
        map.end()
    }
}

mod test {
    #[test]
    fn test_serialize_gnss() {
        use std::str::FromStr;

        use fixed_point::FixedPoint;
        use serde_json::json;

        use super::GNSS;
        use crate::datastructures::measurement::Course;

        let gnss = GNSS { fixed: false, course: Course::from_str("1.1").unwrap() };
        assert_eq!(json!({"fixed": false}), serde_json::to_value(&gnss).unwrap());

        let course: f32 = FixedPoint::<i32, 1>::from_str("1.1").unwrap().into();
        let gnss = GNSS { fixed: true, course: Course::from_str("1.1").unwrap() };
        assert_eq!(json!({"fixed": true, "course": course}), serde_json::to_value(&gnss).unwrap());
    }

    #[test]
    fn test_serialize() {
        use serde_json::json;

        use super::Sensor;

        let expected = json!({
            "acceleration": {
                "axes": {"x": 0, "y": 0, "z": 0},
                "sensitive": 2147483647,
            },
            "gyro": {
                "axes": {"x": 0, "y": 0, "z": 0},
                "sensitive": 2147483647,
            },
            "magnetism": null,
            "gnss": null,
        });
        let sensor = Sensor::default();
        assert_eq!(expected, serde_json::to_value(&sensor).unwrap());
    }
}
