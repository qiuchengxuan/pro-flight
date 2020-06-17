use core::time::Duration;

use embedded_hal::serial::Read;
use embedded_hal::timer::CountDown;

use super::sbus::SbusReceiver;

pub fn probe<E, S, C>(uart: &mut S, count_down: &mut C) -> Result<bool, E>
where
    S: Read<u8, Error = E>,
    C: CountDown<Time = Duration>,
{
    count_down.start(Duration::from_millis(50));
    loop {
        match uart.read() {
            Ok(_) => return Ok(true),
            Err(e) => match e {
                nb::Error::WouldBlock => match count_down.wait() {
                    Ok(_) => return Ok(false),
                    _ => continue,
                },
                nb::Error::Other(e) => return Err(e),
            },
        };
    }
}

pub enum Device {
    None,
    SBUS(SbusReceiver),
    GNSS,
}

pub fn handle(device: &mut Device, ring: &[u8], offset: usize, size: usize) {
    match device {
        Device::SBUS(sbus_device) => sbus_device.handle(ring, offset, size),
        _ => (),
    }
}
