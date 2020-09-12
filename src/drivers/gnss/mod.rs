pub mod nmea;
pub mod ubx;

use crate::config::serial::GNSSProtocol;
use crate::datastructures::coordinate::Position;
use crate::datastructures::data_source::singular::SingularDataSource;
use crate::datastructures::data_source::u16_source::U16DataSource;
use crate::datastructures::measurement::unit::MilliMeter;
use crate::datastructures::measurement::{Course, HeadingOrCourse, VelocityVector};
use crate::datastructures::GNSSFixed;

use nmea::NMEADecoder;
use ubx::UBXDecoder;

pub enum GNSS {
    UBX(UBXDecoder),
    NMEA(NMEADecoder),
}

impl GNSS {
    pub fn new(protocol: GNSSProtocol) -> Self {
        match protocol {
            GNSSProtocol::UBX => Self::UBX(UBXDecoder::new()),
            GNSSProtocol::NMEA => Self::NMEA(NMEADecoder::new()),
        }
    }

    pub fn fixed(&self) -> U16DataSource<GNSSFixed> {
        match self {
            Self::UBX(ubx) => ubx.fixed(),
            Self::NMEA(nmea) => nmea.fixed(),
        }
    }

    pub fn position(&self) -> SingularDataSource<Position> {
        match self {
            Self::UBX(ubx) => ubx.position(),
            Self::NMEA(nmea) => nmea.position(),
        }
    }

    pub fn velocity(&self) -> SingularDataSource<VelocityVector<i32, MilliMeter>> {
        match self {
            Self::UBX(ubx) => ubx.velocity(),
            Self::NMEA(nmea) => nmea.velocity(),
        }
    }

    pub fn heading(&self) -> SingularDataSource<HeadingOrCourse> {
        match self {
            Self::UBX(ubx) => ubx.heading(),
            Self::NMEA(nmea) => nmea.heading(),
        }
    }

    pub fn course(&self) -> SingularDataSource<Course> {
        match self {
            Self::UBX(ubx) => ubx.course(),
            Self::NMEA(nmea) => nmea.course(),
        }
    }

    pub fn handle(&mut self, ring: &[u8], _: bool) {
        match self {
            Self::UBX(ubx) => ubx.handle(ring),
            Self::NMEA(nmea) => nmea.handle(ring),
        }
    }
}
