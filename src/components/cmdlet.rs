use core::fmt::Write;
use core::time::Duration;

use embedded_hal::serial;
use embedded_hal::timer::CountDown;

use crate::components::console::Console;

pub type MemoryAddressValidator = fn(u32) -> bool;

fn no_address_validator(_: u32) -> bool {
    true
}

static mut MEMORY_ADDRESS_VALIDATOR: MemoryAddressValidator = no_address_validator;

pub fn init(validator: MemoryAddressValidator) {
    unsafe { MEMORY_ADDRESS_VALIDATOR = validator };
}

pub fn dump<WE, S: serial::Write<u8, Error = WE>>(line: &str, serial: &mut S) {
    let mut iter = line[5..].split(' ');
    let mut address: u32 = 0;
    if let Some(word) = iter.next() {
        if let Some(addr) = u32::from_str_radix(word, 16).ok() {
            address = addr;
        }
    }
    if address == 0 {
        return;
    }
    if !unsafe { MEMORY_ADDRESS_VALIDATOR }(address) {
        return;
    }
    let mut size: usize = 0;
    if let Some(word) = iter.next() {
        if let Some(sz) = word.parse().ok() {
            size = sz
        }
    }
    let slice = unsafe { core::slice::from_raw_parts(address as *const u8, size) };
    console!(serial, "Result: {:x?}\r\n", slice);
}

pub fn read<WE, S: serial::Write<u8, Error = WE>>(line: &str, serial: &mut S) {
    let mut split = line.split(' ');
    let read = split.next().unwrap_or("read");
    if let Some(address) = split.next().map(|s| u32::from_str_radix(s, 16).ok()).flatten() {
        if unsafe { MEMORY_ADDRESS_VALIDATOR }(address) {
            match read {
                "readx" => {
                    let value = unsafe { *(address as *const u32) };
                    console!(serial, "Result: {:x}\n", value);
                }
                "readf" => {
                    let value = unsafe { *(address as *const f32) };
                    let mut buffer = ryu::Buffer::new();
                    let printed = buffer.format(value);
                    console!(serial, "Result: {}\r\n", printed);
                }
                _ => {
                    let value = unsafe { *(address as *const u32) };
                    console!(serial, "Result: {}\n", value);
                }
            }
        }
    }
}

pub fn write<WE, S, C>(line: &str, serial: &mut S, count_down: &mut C)
where
    S: serial::Write<u8, Error = WE>,
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
                console!(serial, "Write result: {:x?}\r\n", value);
            }
        }
    }
}
