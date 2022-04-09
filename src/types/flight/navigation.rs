use crate::types::{
    coordinate::Position,
    measurement::{unit, VelocityVector},
    waypoint::Steerpoint,
};

#[derive(Copy, Clone, Debug, Default, Serialize)]
pub struct Navigation {
    pub position: Position,
    pub speed_vector: VelocityVector<f32, unit::MpS>,
    pub steerpoint: Steerpoint,
}

mod test {
    #[test]
    fn test_serialize_navigation() {
        use serde_json::json;

        use super::Navigation;

        let expected = json!({
            "position": {
                "latitude": "N00°00'00.000",
                "longitude": "E000°00'00.000",
                "altitude": 0,
            },
            "speed_vector": {"x": 0.0, "y": 0.0, "z": 0.0},
            "steerpoint": {
                "index": 0,
                "waypoint": {
                    "name": "HOME",
                    "position": {
                        "latitude": "N00°00'00.000",
                        "longitude": "E000°00'00.000",
                        "altitude": 0,
                    },
                }
            }
        });
        let nav = Navigation::default();
        assert_eq!(expected, serde_json::to_value(&nav).unwrap());
    }
}
