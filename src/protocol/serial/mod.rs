use alloc::boxed::Box;

use crate::components::flight_data_hub::FlightDataHUB;
use crate::config::peripherals::serial::Config;

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
        Config::SBUS(sbus) => Some(Box::new(SBUS::new(&hub.rssi, sbus.fast, &hub.control_input))),
        Config::GNSS(gnss) => gnss::make_receiver(gnss, hub),
    }
}
