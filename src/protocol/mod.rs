pub mod sbus;
pub mod serial;
pub mod xmodem;

use alloc::boxed::Box;

use crate::components::flight_data::FlightDataHUB;
use crate::config::peripherals::serial::Config;
use sbus::SBUS;
use serial::Receiver;

pub fn make_serial_receiver<'a>(
    config: &Config,
    hub: &'a FlightDataHUB,
) -> Option<Box<dyn Receiver + 'a>> {
    match config {
        Config::SBUS(_) => Some(Box::new(SBUS::new(&hub.rssi, &hub.control_input))),
        _ => None,
    }
}
