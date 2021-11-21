use alloc::boxed::Box;

use crate::{config::peripherals::serial::Config, service::flight::data::FlightDataHUB};

pub trait Receiver: Send {
    fn receive_size(&self) -> usize;
    fn receive(&mut self, bytes: &[u8]);
    fn reset(&mut self);
}

pub mod gnss;
pub mod sbus;

use sbus::SBUS;

pub fn make_receiver<'a>(
    config: &Config,
    hub: &'a FlightDataHUB,
) -> Option<Box<dyn Receiver + 'a>> {
    match config {
        Config::SBUS(sbus) => Some(Box::new(SBUS::new(&hub.rssi, sbus.fast, &hub.input))),
        Config::GNSS(gnss) => gnss::make_receiver(gnss, hub),
    }
}
