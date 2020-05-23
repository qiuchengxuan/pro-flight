use crate::hal::sensors::{Accelerometer, Gyroscope, Thermometer};

pub struct IMU<A, G, T> {
    accelerometer: A,
    gyroscope: G,
    thermometer: T,
}

impl IMU<A: Accelerometer, G: Gyroscope, T: Thermometer> {
    pub fn new(accelerometer: A, gyrosocpe: G, thermometer: T) -> Self {
        Self {
            accelerometer,
            gyroscope,
            thermometer,
        }
    }
}
