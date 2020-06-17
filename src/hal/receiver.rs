pub type Percentage = u8;

pub trait Receiver: core::fmt::Debug {
    fn rssi(&self) -> Percentage;
    fn get_sequence(&self) -> usize;
    fn num_channel(&self) -> usize;
    fn get_channel(&self, index: usize) -> u16;
}

#[derive(Debug)]
pub struct NoReceiver;

impl Receiver for NoReceiver {
    fn rssi(&self) -> Percentage {
        0
    }

    fn get_sequence(&self) -> usize {
        0
    }

    fn num_channel(&self) -> usize {
        0
    }

    fn get_channel(&self, _: usize) -> u16 {
        0
    }
}
