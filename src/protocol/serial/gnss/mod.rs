use alloc::boxed::Box;

use crate::{
    config::peripherals::serial::{GNSSConfig, GNSSProtocol},
    protocol::serial::Receiver,
    service::{flight::data::FlightDataHUB, info::bulletin::Bulletin},
    types::{
        coordinate::Position,
        measurement::{unit, Course, Heading, VelocityVector},
    },
};

pub struct DataSource<'a> {
    pub fixed: &'a Bulletin<bool>,
    pub position: &'a Bulletin<Position>,
    pub velocity: &'a Bulletin<VelocityVector<i32, unit::MMpS>>,
    pub heading: &'a Bulletin<Heading>,
    pub course: &'a Bulletin<Course>,
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
