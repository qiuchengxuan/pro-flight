use alloc::boxed::Box;

use crate::datastructures::data_source::{AgingStaticData, StaticData};
use crate::datastructures::input::ControlInput as Input;

pub struct ControlMixer<S> {
    receiver: Box<dyn AgingStaticData<Input>>,
    receiver_max_age: usize,
    stabilizer: S,
    stablizer_limit: u8,
    // TODO: autopilot
}

fn limit_i16(value: i16, limit: u8) -> i16 {
    let min_value = (i16::MIN as i32 * limit as i32 / 100) as i16;
    let max_value = (i16::MAX as i32 * limit as i32 / 100) as i16;
    if value < min_value {
        min_value
    } else if value > max_value {
        max_value
    } else {
        value
    }
}

impl<S: StaticData<Input>> ControlMixer<S> {
    pub fn new(receiver: Box<dyn AgingStaticData<Input>>, age: usize, stabilizer: S) -> Self {
        Self { receiver, receiver_max_age: age, stabilizer, stablizer_limit: 30 }
    }

    pub fn set_stabilizer_limit(&mut self, limit: u8) {
        self.stablizer_limit = limit;
    }

    pub fn mix(&mut self) -> Input {
        let mut input = self.receiver.read(self.receiver_max_age).unwrap_or_default();
        let stabilizer = self.stabilizer.read();

        if input.roll > 0 && stabilizer.roll < 0 || input.roll < 0 && stabilizer.roll > 0 {
            input.roll += limit_i16(stabilizer.roll, self.stablizer_limit);
        }
        if input.pitch > 0 && stabilizer.pitch < 0 || input.pitch < 0 && stabilizer.pitch > 0 {
            input.pitch += limit_i16(stabilizer.pitch, self.stablizer_limit);
        }
        if input.yaw > 0 && stabilizer.yaw < 0 || input.yaw < 0 && stabilizer.yaw > 0 {
            input.yaw += limit_i16(stabilizer.yaw, self.stablizer_limit);
        }
        input
    }
}
