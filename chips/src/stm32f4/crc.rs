use core::slice::from_raw_parts;

use crc::Hasher32;
use stm32f4xx_hal::pac;

pub struct CRC(pub pac::CRC);

impl Hasher32 for CRC {
    fn reset(&mut self) {
        unsafe { self.0.cr.write(|w| w.bits(1)) }
    }

    fn write(&mut self, bytes: &[u8]) {
        let size = bytes.len() / 4;
        let u32s: &[u32] = unsafe { from_raw_parts(bytes.as_ptr() as *const u32, size) };
        for &data in u32s.iter() {
            unsafe { self.0.dr.write(|w| w.bits(u32::from_be(data))) }
        }
        for &b in bytes[size * 4..].iter() {
            unsafe { self.0.dr.write(|w| w.bits(b as u32)) }
        }
    }

    fn sum32(&self) -> u32 {
        self.0.dr.read().bits()
    }
}
