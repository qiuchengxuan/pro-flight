use chrono::naive::NaiveDateTime;

use crate::types::{
    coordinate::Position,
    measurement::{unit, Course, Heading, Velocity, VelocityVector, ENU},
};

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Fixed {
    pub position: Position,
    pub course: Course,
    #[serde(rename = "ground-speed")]
    pub ground_speed: Velocity<i32, unit::MMs>,
    #[serde(rename = "velocity-vector")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub velocity_vector: Option<VelocityVector<i32, unit::MMs, ENU>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<Heading>,
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct GNSS {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed: Option<Fixed>,
}

mod test {
    #[test]
    fn test_serialize_gnss() {
        use std::str::FromStr;

        use fixed_point::FixedPoint;
        use serde_json::json;

        use super::{Fixed, GNSS};
        use crate::types::measurement::Course;

        let gnss = GNSS { datetime: None, fixed: None };
        assert_eq!(json!({}), serde_json::to_value(&gnss).unwrap());

        let course: f32 = FixedPoint::<i32, 1>::from_str("1.1").unwrap().into();
        let gnss = GNSS {
            datetime: None,
            fixed: Some(Fixed {
                course: Course(FixedPoint::from_str("1.1").unwrap()),
                ..Default::default()
            }),
        };
        let expected = json!({
            "fixed": {
                "course": course,
                "ground-speed": 0,
                "position": {
                    "altitude": 0,
                    "latitude": "N00°00'00.000",
                    "longitude": "E000°00'00.000",
                }
            }
        });

        assert_eq!(expected, serde_json::to_value(&gnss).unwrap());
    }
}
