use super::controller::{ControlSurfaceInput, Controller, ThrottleInput};

pub type Percentage = u8;

pub trait Receiver: Controller {
    fn rssi(&self) -> u8;
    fn get_sequence(&self) -> usize;
}

pub struct NoReceiver;

impl Controller for NoReceiver {
    fn get_throttle(&self) -> ThrottleInput {
        ThrottleInput::default()
    }

    fn get_input(&self) -> ControlSurfaceInput {
        ControlSurfaceInput::default()
    }
}

impl Receiver for NoReceiver {
    fn rssi(&self) -> Percentage {
        0
    }

    fn get_sequence(&self) -> usize {
        0
    }
}
