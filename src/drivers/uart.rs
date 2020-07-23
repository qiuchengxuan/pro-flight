use super::sbus::SbusReceiver;

pub enum Device {
    None,
    SBUS(SbusReceiver),
    GNSS,
}

impl Device {
    pub fn handle(&mut self, ring: &[u8], offset: usize, size: usize) {
        match self {
            Device::SBUS(sbus_device) => sbus_device.handle(ring, offset, size),
            _ => (),
        }
    }
}
