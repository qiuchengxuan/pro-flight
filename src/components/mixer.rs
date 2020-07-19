use crate::datastructures::data_source::DataSource;
use crate::datastructures::input::ControlInput;

pub struct ControlMixer<S> {
    receiver: &'static mut dyn DataSource<ControlInput>,
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

impl<S: DataSource<ControlInput>> ControlMixer<S> {
    pub fn new(receiver: &'static mut dyn DataSource<ControlInput>, stabilizer: S) -> Self {
        Self { receiver, stabilizer, stablizer_limit: 30 }
    }

    pub fn set_stabilizer_limit(&mut self, limit: u8) {
        self.stablizer_limit = limit;
    }

    pub fn mix(&mut self) -> ControlInput {
        let mut input = self.receiver.read_last_unchecked();
        let stabilizer = self.stabilizer.read_last_unchecked();

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
