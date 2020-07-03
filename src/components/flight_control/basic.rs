use crate::datastructures::input::{Pitch, Roll, Throttle, Yaw};

pub trait BasicControl {
    fn set(&mut self, throttle: Throttle, roll: Roll, pitch: Pitch, yaw: Yaw);
}
