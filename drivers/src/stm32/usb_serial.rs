use alloc::boxed::Box;
use core::mem::MaybeUninit;

use drone_core::prelude::*;
use embedded_hal::blocking::delay::DelayUs;
use pro_flight::sys::timer::SysTimer;
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, UsbError};

static mut USB_DEVICE: MaybeUninit<UsbDevice<'static, UsbBus<USB>>> = MaybeUninit::uninit();
static mut SERIAL_PORT: Option<SerialPort<'static, UsbBus<USB>>> = None;

fn poll() -> bool {
    let device = unsafe { &mut *USB_DEVICE.as_mut_ptr() };
    let serial_port = unsafe { SERIAL_PORT.as_mut() }.unwrap();
    cortex_m::interrupt::free(|_| device.poll(&mut [serial_port]))
}

fn flush() -> bool {
    let mut delay = SysTimer::new();
    for _ in 0..4 {
        if poll() {
            return true;
        }
        delay.delay_us(250u32);
    }
    false
}

fn write_bytes(mut bytes: &[u8]) {
    let serial_port = match unsafe { SERIAL_PORT.as_mut() } {
        Some(port) => port,
        None => return,
    };

    while bytes.len() > 0 {
        match cortex_m::interrupt::free(|_| serial_port.write(bytes)) {
            Ok(size) => bytes = &bytes[size..],
            Err(UsbError::WouldBlock) => {
                if !flush() {
                    return;
                }
            }
            Err(_) => return,
        }
    }
}

#[no_mangle]
fn drone_log_is_enabled(_port: u8) -> bool {
    unsafe { SERIAL_PORT.is_some() }
}

#[no_mangle]
fn drone_log_write_bytes(_port: u8, bytes: &[u8]) {
    write_bytes(bytes)
}

#[no_mangle]
fn drone_log_write_u8(_port: u8, value: u8) {
    write_bytes(&value.to_be_bytes())
}

#[no_mangle]
fn drone_log_write_u16(_port: u8, value: u16) {
    write_bytes(&value.to_be_bytes())
}

#[no_mangle]
fn drone_log_write_u32(_port: u8, value: u32) {
    write_bytes(&value.to_be_bytes())
}

#[no_mangle]
fn drone_log_flush() {
    if unsafe { SERIAL_PORT.is_some() } {
        poll();
    }
}

pub fn read(buffer: &mut [u8]) -> &[u8] {
    cortex_m::interrupt::free(move |_| {
        let serial_port = unsafe { SERIAL_PORT.as_mut() }.unwrap();
        let size = serial_port.read(buffer).ok().unwrap_or(0);
        &buffer[..size]
    })
}

type Allocator = UsbBusAllocator<UsbBus<USB>>;

pub fn init(alloc: Allocator, board_name: &'static str) -> impl Fn() -> bool {
    let allocator: &'static mut Allocator = Box::leak(Box::new(alloc));
    unsafe { SERIAL_PORT = Some(SerialPort::new(allocator)) }
    let device = UsbDeviceBuilder::new(allocator, UsbVidPid(0x0403, 0x6001))
        .product(board_name)
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();
    unsafe { USB_DEVICE = MaybeUninit::new(device) }

    poll
}
