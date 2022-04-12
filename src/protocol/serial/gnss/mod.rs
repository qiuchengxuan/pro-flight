use crate::{
    config::peripherals::serial::GNSSProtocol as Protocol, datastore, protocol::serial::Receiver,
};

pub mod nmea;
pub mod out;
pub mod ubx;

pub enum GNSSReceiver {
    UBX(ubx::UBX),
    NMEA(nmea::NMEA),
}

impl Receiver for GNSSReceiver {
    fn chunk_size(&self) -> usize {
        match self {
            Self::UBX(_) => ubx::CHUNK_SIZE,
            Self::NMEA(_) => nmea::CHUNK_SIZE,
        }
    }

    fn receive(&mut self, bytes: &[u8]) {
        let gnss = match self {
            Self::UBX(ref mut ubx) => ubx.receive(bytes),
            Self::NMEA(ref mut nmea) => nmea.receive(bytes),
        };
        if let Some(gnss) = gnss {
            datastore::acquire().write_gnss(gnss);
        }
    }

    fn reset(&mut self) {
        match self {
            Self::UBX(ref mut ubx) => ubx.reset(),
            Self::NMEA(ref mut nmea) => nmea.reset(),
        }
    }
}

impl From<Protocol> for GNSSReceiver {
    fn from(protocol: Protocol) -> Self {
        match protocol {
            Protocol::NMEA => GNSSReceiver::NMEA(nmea::NMEA::new()),
            Protocol::UBX => GNSSReceiver::UBX(ubx::UBX::new()),
        }
    }
}
