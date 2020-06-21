pub type Percentage = u8;

#[derive(Default, Value)]
pub struct ReceiverInput {
    pub throttle: u16,
    pub roll: i16,
    pub pitch: i16,
    pub yaw: i16,
}

impl core::fmt::Display for ReceiverInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval_json::to_fmt(f, self).ok();
        Ok(())
    }
}

pub trait Receiver {
    fn rssi(&self) -> u8;
    fn get_sequence(&self) -> usize;
    fn get_input(&self) -> ReceiverInput;
}

pub struct NoReceiver;

impl Receiver for NoReceiver {
    fn rssi(&self) -> Percentage {
        0
    }

    fn get_sequence(&self) -> usize {
        0
    }

    fn get_input(&self) -> ReceiverInput {
        ReceiverInput::default()
    }
}
