pub mod dfu;

pub fn valid_memory_address(address: u32) -> bool {
    match address {
        0x40000000..=0xA0000FFF => true,
        0x20000000..=0x2001FFFF => true,
        0x10000000..=0x1000FFFF => true,
        _ => false,
    }
}
