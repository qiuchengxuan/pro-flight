use alloc::boxed::Box;

use crate::components::flight_data::FlightDataHUB;
use crate::config::peripherals::serial::{GNSSConfig, GNSSProtocol};
use crate::datastructures::{
    coordinate::Position,
    measurement::{unit, Course, Heading, VelocityVector},
    GNSSFixed,
};
use crate::protocol::serial::Receiver;
use crate::sync::singular::SingularData;

pub struct DataSource<'a> {
    pub fixed: &'a SingularData<GNSSFixed>,
    pub position: &'a SingularData<Position>,
    pub velocity: &'a SingularData<VelocityVector<i32, unit::MMpS>>,
    pub heading: &'a SingularData<Heading>,
    pub course: &'a SingularData<Course>,
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
