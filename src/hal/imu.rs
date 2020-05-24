use crate::datastructures::event::EventHandler;
use crate::hal::sensors::{Acceleration, Gyro};

pub type AccelGyroHandler = EventHandler<(&'static [Acceleration<u32>], &'static [Gyro<u32>])>;

pub struct Attitude {
    pub pitch: i8, // negative means sink
    pub roll: i8,  // (-90, 90], clock wise
    pub yaw: u16,  // ref to north
}

impl Default for Attitude {
    fn default() -> Self {
        Self {
            pitch: 0,
            roll: 0,
            yaw: 0,
        }
    }
}

pub trait IMU {
    fn get_attitude(&self) -> Attitude;
}
