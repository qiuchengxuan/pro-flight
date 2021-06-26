use alloc::boxed::Box;

use crate::components::flight_data::FlightDataHUB;
use crate::config::peripherals::serial::Config;

pub trait Receiver: Send {
    fn receive(&mut self, bytes: &[u8]);
}

pub mod gnss;
pub mod sbus;

use sbus::SBUS;

pub fn make_receiver<'a>(
    config: &Config,
    hub: &'a FlightDataHUB,
) -> Option<Box<dyn Receiver + 'a>> {
    match config {
        Config::SBUS(_) => Some(Box::new(SBUS::new(&hub.rssi, &hub.control_input))),
        Config::GNSS(config) => gnss::make_receiver(config, hub),
    }
}
