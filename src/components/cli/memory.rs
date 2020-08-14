use core::time::Duration;

use embedded_hal::timer::CountDown;

pub type MemoryAddressValidator = fn(u32) -> bool;

fn no_address_validator(_: u32) -> bool {
    true
}

static mut MEMORY_ADDRESS_VALIDATOR: MemoryAddressValidator = no_address_validator;

pub fn init(validator: MemoryAddressValidator) {
    unsafe { MEMORY_ADDRESS_VALIDATOR = validator };
}

pub fn dump<W: core::fmt::Write>(line: &str, w: &mut W) -> core::fmt::Result {
    let mut iter = line[5..].split(' ');
    let mut address: u32 = 0;
    if let Some(word) = iter.next() {
        if let Some(addr) = u32::from_str_radix(word, 16).ok() {
            address = addr;
        }
    }
    if address == 0 {
        return Ok(());
    }
    if !unsafe { MEMORY_ADDRESS_VALIDATOR }(address) {
        return Ok(());
    }
    let mut size: usize = 0;
    if let Some(word) = iter.next() {
        if let Some(sz) = word.parse().ok() {
            size = sz
        }
    }
    let slice = unsafe { core::slice::from_raw_parts(address as *const u8, size) };
    writeln!(w, "Result: {:x?}", slice)
}

pub fn read<W: core::fmt::Write>(line: &str, w: &mut W) -> core::fmt::Result {
    let mut split = line.split(' ');
    let read = split.next().unwrap_or("read");
    if let Some(address) = split.next().map(|s| u32::from_str_radix(s, 16).ok()).flatten() {
        if unsafe { MEMORY_ADDRESS_VALIDATOR }(address) {
            return match read {
                "readx" => {
                    let value = unsafe { *(address as *const u32) };
                    writeln!(w, "Result: {:x}", value)
                }
                _ => {
                    let value = unsafe { *(address as *const u32) };
                    writeln!(w, "Result: {}", value)
                }
            };
        }
    }
    Ok(())
}

pub fn write<W, C>(line: &str, w: &mut W, count_down: &mut C) -> core::fmt::Result
where
    W: core::fmt::Write,
    C: CountDown<Time = Duration>,
{
    let mut iter = line[6..].split(' ').flat_map(|w| u32::from_str_radix(w, 16).ok());
    if let Some(address) = iter.next() {
        if let Some(value) = iter.next() {
            if unsafe { MEMORY_ADDRESS_VALIDATOR }(address) {
                unsafe { *(address as *mut u32) = value };
                count_down.start(Duration::from_millis(1));
                nb::block!(count_down.wait()).unwrap();
                let value = unsafe { *(address as *const u32) };
                return writeln!(w, "Write result: {:x?}", value);
            }
        }
    }
    Ok(())
}
