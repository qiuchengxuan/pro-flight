use super::gnss::GNSS;
use super::sbus::SbusReceiver;

pub enum Device {
    SBUS(SbusReceiver),
    GNSS(GNSS),
}

impl Device {
    pub fn handle(&mut self, ring: &[u8], half: bool) {
        match self {
            Device::SBUS(sbus_device) => sbus_device.handle(ring, half),
            Device::GNSS(gnss) => gnss.handle(ring, half),
        }
    }
}
