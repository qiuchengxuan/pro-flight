pub mod ubx;

use crate::config::serial::GNSSProtocol;
use crate::datastructures::coordinate::Position;
use crate::datastructures::data_source::DataSource;

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

    pub fn as_position_source(&mut self) -> impl DataSource<Position> {
        match self {
            Self::UBX(ubx) => ubx.as_position_source(),
        }
    }

    pub fn handle(&mut self, ring: &[u8], half: bool) {
        match self {
            Self::UBX(ubx) => ubx.handle(ring, half),
        }
    }
}
