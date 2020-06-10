use crate::hal::sensors::Acceleration;

#[derive(Value)]
pub struct Config {
    pub accel_calibration: Acceleration,
}
