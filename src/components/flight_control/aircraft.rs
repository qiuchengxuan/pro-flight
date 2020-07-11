use super::basic::BasicControl;
use crate::datastructures::schedule::Schedulable;
use crate::hal::input::{BasicInput, NoInput};

pub struct Aircraft<'a, B> {
    aircraft: B,
    receiver: &'a dyn BasicInput,
    stablizer: &'a dyn BasicInput,
    stablizer_limit: u8,
    autopilot: &'a dyn BasicInput,
    autopilot_limit: u8,
}

const NO_INPUT: NoInput = NoInput {};

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

impl<'a, B: BasicControl> Aircraft<'a, B> {
    pub fn new(aircraft: B, receiver: &'a dyn BasicInput) -> Self {
        Self {
            aircraft,
            receiver,
            stablizer: &NO_INPUT,
            autopilot: &NO_INPUT,
            stablizer_limit: 30,
            autopilot_limit: 30,
        }
    }

    pub fn set_stablizer_limit(&mut self, limit: u8) {
        self.stablizer_limit = limit;
    }

    pub fn set_flight_controller_limit(&mut self, limit: u8) {
        self.autopilot_limit = limit;
    }
}

impl<'a, B: BasicControl> Schedulable for Aircraft<'a, B> {
    fn schedule(&mut self) {
        let throttle = self.receiver.get_throttle();

        let stablizer_roll = limit_i16(self.stablizer.get_roll(), self.stablizer_limit);
        let autopilot_roll = limit_i16(self.autopilot.get_roll(), self.stablizer_limit);
        let roll = self.receiver.get_roll() + stablizer_roll + autopilot_roll;

        let stablizer_pitch = limit_i16(self.stablizer.get_pitch(), self.stablizer_limit);
        let autopilot_pitch = limit_i16(self.autopilot.get_pitch(), self.stablizer_limit);
        let pitch = self.receiver.get_pitch() + stablizer_pitch + autopilot_pitch;

        let stablizer_yaw = limit_i16(self.stablizer.get_yaw(), self.stablizer_limit);
        let autopilot_yaw = limit_i16(self.autopilot.get_yaw(), self.stablizer_limit);
        let yaw = self.receiver.get_yaw() + stablizer_yaw + autopilot_yaw;

        self.aircraft.set(throttle, roll, pitch, yaw);
    }
}
