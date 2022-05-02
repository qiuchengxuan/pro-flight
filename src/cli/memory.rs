use embedded_hal::blocking::delay::DelayMs;

use crate::sys::time::TickTimer;

extern "Rust" {
    fn memory_valid_address(address: usize) -> bool;
}

#[macro_export]
macro_rules! no_memory_valid_address {
    () => {
        #[no_mangle]
        fn memory_valid_address(address: usize) -> bool {
            false
        }
    };
}

pub fn dump(line: &str) {
    let (address, size) =
        line.split_once(' ').map(|(a, b)| (a, b.trim_start())).unwrap_or((line, ""));
    let address: usize = usize::from_str_radix(address, 16).unwrap_or(0);
    if address == 0 {
        return;
    }
    if !unsafe { memory_valid_address(address) } {
        return;
    }
    let size: usize = size.parse().unwrap_or(0);
    let slice = unsafe { core::slice::from_raw_parts(address as *const u8, size) };
    println!("Result: {:02x?}", slice)
}

fn _read(line: &str) -> Option<usize> {
    if let Some(address) = usize::from_str_radix(line.trim_start(), 16).ok() {
        if unsafe { memory_valid_address(address) } {
            return Some(unsafe { *(address as *const usize) });
        }
    }
    None
}

pub fn read(line: &str) {
    if let Some(value) = _read(line) {
        println!("Result: {}", value)
    }
}

pub fn readx(line: &str) {
    if let Some(value) = _read(line) {
        println!("Result: {:x}", value);
    }
}

pub fn writex(line: &str) {
    let (address, value) =
        line.split_once(' ').map(|(a, b)| (a, b.trim_start())).unwrap_or((line, ""));
    let (address, value) =
        match (usize::from_str_radix(address, 16), usize::from_str_radix(value, 16)) {
            (Ok(address), Ok(value)) => (address, value),
            _ => return,
        };
    if unsafe { memory_valid_address(address) } {
        unsafe { core::ptr::write_volatile(address as *mut usize, value) };
        TickTimer::default().delay_ms(1u32);
        let value = unsafe { core::ptr::read_volatile(address as *const usize) };
        println!("Write result: {:x?}", value);
    }
}
