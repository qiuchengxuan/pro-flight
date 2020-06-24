use super::controller::{ControlInput, Controller};

pub type Percentage = u8;

pub trait Receiver: Controller {
    fn rssi(&self) -> u8;
    fn get_sequence(&self) -> usize;
}

pub struct NoReceiver;

impl Controller for NoReceiver {
    fn get_input(&self) -> ControlInput {
        ControlInput::default()
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
