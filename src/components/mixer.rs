use crate::datastructures::input::ControlInput as Input;
use crate::sync::{AgingDataReader, DataReader};

pub struct ControlMixer<R, S> {
    receiver: R,
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

impl<R: AgingDataReader<Input>, S: DataReader<Input>> ControlMixer<R, S> {
    pub fn new(receiver: R, stabilizer: S, age: usize) -> Self {
        Self { receiver, receiver_max_age: age, stabilizer, stablizer_limit: 30 }
    }

    pub fn set_stabilizer_limit(&mut self, limit: u8) {
        self.stablizer_limit = limit;
    }

    pub fn mix(&mut self) -> Input {
        let mut input = self.receiver.get_aging_last(self.receiver_max_age).unwrap_or_default();
        if let Some(stabilizer) = self.stabilizer.get_last() {
            input.roll += limit_i16(stabilizer.roll, self.stablizer_limit);
            input.pitch += limit_i16(stabilizer.pitch, self.stablizer_limit);
            input.yaw += limit_i16(stabilizer.yaw, self.stablizer_limit);
        }
        input
    }
}
