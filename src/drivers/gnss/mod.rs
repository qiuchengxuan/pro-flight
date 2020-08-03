pub mod ubx;

use crate::config::serial::GNSSProtocol;
use crate::datastructures::coordinate::Position;
use crate::datastructures::data_source::DataSource;
use crate::datastructures::measurement::{Course, Heading, Velocity};

use ubx::UBXDecoder;

pub enum GNSS {
    UBX(UBXDecoder),
}

impl GNSS {
    pub fn new(protocol: GNSSProtocol) -> Self {
        match protocol {
            GNSSProtocol::UBX => Self::UBX(UBXDecoder::new()),
        }
    }

    pub fn position(&self) -> impl DataSource<Position> {
        match self {
            Self::UBX(ubx) => ubx.position(),
        }
    }

    pub fn velocity(&self) -> impl DataSource<[Velocity<i32>; 3]> {
        match self {
            Self::UBX(ubx) => ubx.velocity(),
        }
    }

    pub fn heading(&self) -> impl DataSource<Heading> {
        match self {
            Self::UBX(ubx) => ubx.heading(),
        }
    }

    pub fn course(&self) -> impl DataSource<Course> {
        match self {
            Self::UBX(ubx) => ubx.course(),
        }
    }

    pub fn handle(&mut self, ring: &[u8], half: bool) {
        match self {
            Self::UBX(ubx) => ubx.handle(ring, half),
        }
    }
}
