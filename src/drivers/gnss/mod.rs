pub mod ubx;

use crate::config::serial::GNSSProtocol;
use crate::datastructures::coordinate::Position;
use crate::datastructures::data_source::singular::SingularDataSource;
use crate::datastructures::gnss::FixType;
use crate::datastructures::measurement::unit::MilliMeter;
use crate::datastructures::measurement::{Course, HeadingOrCourse, VelocityVector};

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

    pub fn fix_type(&self) -> SingularDataSource<FixType> {
        match self {
            Self::UBX(ubx) => ubx.fix_type(),
        }
    }

    pub fn position(&self) -> SingularDataSource<Position> {
        match self {
            Self::UBX(ubx) => ubx.position(),
        }
    }

    pub fn velocity(&self) -> SingularDataSource<VelocityVector<i32, MilliMeter>> {
        match self {
            Self::UBX(ubx) => ubx.velocity(),
        }
    }

    pub fn heading(&self) -> SingularDataSource<HeadingOrCourse> {
        match self {
            Self::UBX(ubx) => ubx.heading(),
        }
    }

    pub fn course(&self) -> SingularDataSource<Course> {
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
