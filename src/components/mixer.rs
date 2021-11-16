use crate::{
    config::aircraft::Configuration,
    datastructures::{control::Control, output::Output},
    sync::{cell::CellReader, AgingDataReader},
};

pub struct ControlMixer<'a> {
    receiver: CellReader<'a, Control>,
    receiver_max_age: usize,
    configuration: Configuration,
}

impl<'a> ControlMixer<'a> {
    pub fn new(receiver: CellReader<'a, Control>, age: usize) -> Self {
        Self {
            receiver,
            receiver_max_age: age,
            configuration: crate::config::get().aircraft.configuration,
        }
    }

    pub fn mix(&mut self) -> Output {
        let input = self.receiver.get_aging_last(self.receiver_max_age).unwrap_or_default();
        Output::from(&input, self.configuration)
    }
}
