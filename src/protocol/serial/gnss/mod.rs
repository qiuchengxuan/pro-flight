use alloc::boxed::Box;

use crate::{
    components::flight_data_hub::FlightDataHUB,
    config::peripherals::serial::{GNSSConfig, GNSSProtocol},
    datastructures::{
        coordinate::Position,
        measurement::{unit, Course, Heading, VelocityVector},
    },
    protocol::serial::Receiver,
    sync::cell::Cell,
};

pub struct DataSource<'a> {
    pub fixed: &'a Cell<bool>,
    pub position: &'a Cell<Position>,
    pub velocity: &'a Cell<VelocityVector<i32, unit::MMpS>>,
    pub heading: &'a Cell<Heading>,
    pub course: &'a Cell<Course>,
}

pub mod nmea;
pub mod ubx;

use nmea::NMEA;
use ubx::UBX;

pub fn make_receiver<'a>(
    config: &GNSSConfig,
    hub: &'a FlightDataHUB,
) -> Option<Box<dyn Receiver + 'a>> {
    let data_source = DataSource {
        fixed: &hub.gnss_fixed,
        position: &hub.gnss_position,
        velocity: &hub.gnss_velocity,
        heading: &hub.gnss_heading,
        course: &hub.gnss_course,
    };
    match config.protocol {
        GNSSProtocol::NMEA => Some(Box::new(NMEA::new(data_source))),
        GNSSProtocol::UBX => Some(Box::new(UBX::new(data_source))),
    }
}
