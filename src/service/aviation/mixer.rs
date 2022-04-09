use crate::{
    service::info::{bulletin::BulletinReader, AgingReader},
    types::control::Control,
};

pub struct ControlMixer<'a> {
    receiver: BulletinReader<'a, Control>,
    receiver_max_age: usize,
}

impl<'a> ControlMixer<'a> {
    pub fn new(receiver: BulletinReader<'a, Control>, age: usize) -> Self {
        Self { receiver, receiver_max_age: age }
    }

    pub fn mix(&mut self) -> Control {
        self.receiver.get_aging_last(self.receiver_max_age).unwrap_or_default()
    }
}
