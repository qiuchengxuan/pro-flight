use serde::ser::SerializeStruct;

use crate::types::measurement::{Acceleration, Course, Gyro, Magnetism};

#[derive(Copy, Clone, Debug)]
pub struct GNSS {
    pub fixed: bool,
    pub course: Course,
}

impl serde::Serialize for GNSS {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut struct_ = serializer.serialize_struct("GNSS", 2)?;
        struct_.serialize_field("fixed", &self.fixed)?;
        if self.fixed {
            struct_.serialize_field("course", &self.course)?;
        }
        struct_.end()
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
        let mut struct_ = serializer.serialize_struct("Sensor", 4)?;
        struct_.serialize_field("acceleration", &self.acceleration)?;
        struct_.serialize_field("gyro", &self.gyro)?;
        struct_.serialize_field("magnetism", &self.magnetism)?;
        struct_.serialize_field("gnss", &self.gnss)?;
        struct_.end()
    }
}

mod test {
    #[test]
    fn test_serialize_gnss() {
        use std::str::FromStr;

        use fixed_point::FixedPoint;
        use serde_json::json;

        use super::GNSS;
        use crate::types::measurement::Course;

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
            "acceleration": {"x": 0.0, "y": 0.0, "z": 0.0},
            "gyro": {"x": 0.0, "y": 0.0, "z": 0.0},
            "magnetism": null,
            "gnss": null,
        });
        let sensor = Sensor::default();
        assert_eq!(expected, serde_json::to_value(&sensor).unwrap());
    }
}
