#[cfg(feature = "stm32f4xx-hal")]
pub mod crc;
pub mod dfu;

pub fn valid_memory_address(address: u32) -> bool {
    match address {
        0xE000E008..=0xE000EF44 => true,
        0x40000000..=0xA0000FFF => true,
        0x20000000..=0x2001FFFF => true,
        0x10000000..=0x1000FFFF => true,
        _ => false,
    }
}
