use crate::{
    datastructures::control::Control,
    sync::{cell::CellReader, AgingDataReader},
};

pub struct ControlMixer<'a> {
    receiver: CellReader<'a, Control>,
    receiver_max_age: usize,
}

impl<'a> ControlMixer<'a> {
    pub fn new(receiver: CellReader<'a, Control>, age: usize) -> Self {
        Self { receiver, receiver_max_age: age }
    }

    pub fn mix(&mut self) -> Control {
        self.receiver.get_aging_last(self.receiver_max_age).unwrap_or_default()
    }
}
