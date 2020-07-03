pub type Percentage = u8;

pub trait Receiver {
    fn rssi(&self) -> u8;
    fn get_sequence(&self) -> usize;
}

pub struct NoReceiver;

impl Receiver for NoReceiver {
    fn rssi(&self) -> Percentage {
        0
    }

    fn get_sequence(&self) -> usize {
        0
    }
}
