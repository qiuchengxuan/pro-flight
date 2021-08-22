pub mod aviation;
pub mod misc;
pub mod navigation;
pub mod sensor;

#[derive(Copy, Clone, Default, Debug, Serialize)]
pub struct FlightData {
    pub aviation: aviation::Aviation,
    pub navigation: navigation::Navigation,
    pub sensor: sensor::Sensor,
    pub misc: misc::Misc,
}

impl core::fmt::Display for FlightData {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        serde_json_core_fmt::to_fmt(f, self)
    }
}

mod test {
    #[test]
    fn test_flight_data() {
        use serde_json::json;

        use super::FlightData;

        let expected = json!({
            "aviation": {
                "altitude": 0,
                "attitude": {
                    "roll": 0,
                    "pitch": 0
                },
                "heading": 0,
                "height": 0,
                "g_force": 0,
                "airspeed": 0,
                "vario": 0
            },
            "navigation": {
                "position": {
                    "latitude": "N00°00'00.000",
                    "longitude": "E000°00'00.000",
                    "altitude": 0
                },
                "speed_vector": {"x": 0.0, "y": 0.0, "z": 0.0},
                "steerpoint": {
                    "index": 0,
                    "waypoint": {
                        "name": "HOME",
                        "position": {
                            "latitude": "N00°00'00.000",
                            "longitude": "E000°00'00.000",
                            "altitude": 0
                        }
                    }
                }
            },
            "sensor": {
                "acceleration": {"x": 0.0, "y": 0.0, "z": 0.0},
                "gyro": {"x": 0.0, "y": 0.0, "z": 0.0},
                "magnetism": null,
                "gnss": null
            },
            "misc": {
                "battery": 0,
                "displacement": {"x": 0, "y": 0, "z": 0},
                "input": {
                    "throttle": 0,
                    "roll": 0,
                    "pitch": 0,
                    "yaw": 0
                },
                "quaternion": [0.0, 0.0, 0.0, 1.0],
                "rssi": 0
            }
        });
        let data = FlightData::default();
        let string = format!("{}", data);
        assert_eq!(expected, serde_json::from_str::<serde_json::Value>(&string).unwrap());
    }
}
