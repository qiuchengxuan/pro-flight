use alloc::boxed::Box;
use core::sync::atomic::{AtomicUsize, Ordering};

use drone_core::prelude::*;
use embedded_hal::blocking::delay::DelayUs;
use pro_flight::sys::timer::SysTimer;
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, UsbError};

static mut SERIAL_PORT: Option<SerialPort<'static, UsbBus<USB>>> = None;
static SERIAL_FAIL: AtomicUsize = AtomicUsize::new(4);

fn write_bytes(mut bytes: &[u8]) {
    let serial_port = match unsafe { SERIAL_PORT.as_mut() } {
        Some(port) => port,
        None => return,
    };

    let mut delay = SysTimer::new();
    while bytes.len() > 0 && SERIAL_FAIL.load(Ordering::Relaxed) < 4 {
        match cortex_m::interrupt::free(|_| serial_port.write(bytes)) {
            Ok(size) => bytes = &bytes[size..],
            Err(UsbError::WouldBlock) => {
                delay.delay_us(250u32);
                SERIAL_FAIL.fetch_add(1, Ordering::Relaxed);
            }
            Err(_) => return,
        }
    }
}

#[no_mangle]
extern "C" fn drone_log_is_enabled(_port: u8) -> bool {
    unsafe { SERIAL_PORT.is_some() }
}

#[no_mangle]
extern "C" fn drone_log_write_bytes(_port: u8, buffer: *const u8, count: usize) {
    write_bytes(unsafe { core::slice::from_raw_parts(buffer, count) })
}

#[no_mangle]
extern "C" fn drone_log_write_u8(_port: u8, value: u8) {
    write_bytes(&value.to_be_bytes())
}

#[no_mangle]
extern "C" fn drone_log_write_u16(_port: u8, value: u16) {
    write_bytes(&value.to_be_bytes())
}

#[no_mangle]
extern "C" fn drone_log_write_u32(_port: u8, value: u32) {
    write_bytes(&value.to_be_bytes())
}

#[no_mangle]
extern "C" fn drone_log_flush() {
    if let Some(serial_port) = unsafe { SERIAL_PORT.as_mut() } {
        cortex_m::interrupt::free(|_| serial_port.flush().ok());
    }
}

pub fn read(buffer: &mut [u8]) -> &[u8] {
    cortex_m::interrupt::free(move |_| {
        let serial_port = unsafe { SERIAL_PORT.as_mut() }.unwrap();
        let size = serial_port.read(buffer).ok().unwrap_or(0);
        &buffer[..size]
    })
}

pub struct UsbPoller(UsbDevice<'static, UsbBus<USB>>);

impl UsbPoller {
    pub fn poll(&mut self) {
        cortex_m::interrupt::free(|_| {
            let serial_port = unsafe { SERIAL_PORT.as_mut() }.unwrap();
            if self.0.poll(&mut [serial_port]) {
                SERIAL_FAIL.store(0, Ordering::Relaxed);
            }
        });
    }
}

type Allocator = UsbBusAllocator<UsbBus<USB>>;

pub fn init(alloc: Allocator, board_name: &'static str) -> UsbPoller {
    let allocator: &'static mut Allocator = Box::leak(Box::new(alloc));
    unsafe { SERIAL_PORT = Some(SerialPort::new(allocator)) }
    let device = UsbDeviceBuilder::new(allocator, UsbVidPid(0x0403, 0x6001))
        .product(board_name)
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();
    UsbPoller(device)
}
