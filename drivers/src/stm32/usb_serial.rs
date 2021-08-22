use alloc::boxed::Box;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};

use pro_flight::io::Error;
use stm32f4xx_hal::{
    otg_fs::{UsbBus, USB},
    stm32,
};
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, UsbError};

static mut USB_DEVICE: MaybeUninit<UsbDevice<'static, UsbBus<USB>>> = MaybeUninit::uninit();
static mut SERIAL_PORT: Option<SerialPort<'static, UsbBus<USB>>> = None;
static SUSPEND: AtomicBool = AtomicBool::new(true);
static RX_FULL: AtomicBool = AtomicBool::new(false);

unsafe fn poll() {
    if RX_FULL.fetch_or(true, Ordering::Relaxed) {
        cortex_m::peripheral::NVIC::mask(stm32::Interrupt::OTG_FS)
    }
    let device = &mut *USB_DEVICE.as_mut_ptr();
    let serial_port = SERIAL_PORT.as_mut().unwrap();
    device.poll(&mut [serial_port]);
    SUSPEND.store(device.state() == UsbDeviceState::Suspend, Ordering::Relaxed);
}

#[no_mangle]
fn stdout_flush() {
    if SUSPEND.load(Ordering::Relaxed) {
        return;
    }

    let serial_port = match unsafe { SERIAL_PORT.as_mut() } {
        Some(port) => port,
        None => return,
    };

    loop {
        match cortex_m::interrupt::free(|_| serial_port.flush()) {
            Err(UsbError::WouldBlock) => continue,
            _ => return,
        }
    }
}

#[no_mangle]
fn stdout_write_bytes(bytes: &[u8]) -> usize {
    let serial_port = match unsafe { SERIAL_PORT.as_mut() } {
        Some(port) => port,
        None => return 0,
    };

    let mut written = 0;
    while !SUSPEND.load(Ordering::Relaxed) && written < bytes.len() {
        match cortex_m::interrupt::free(|_| serial_port.write(&bytes[written..])) {
            Ok(size) => written += size,
            Err(UsbError::WouldBlock) => stdout_flush(),
            Err(_) => return written,
        }
    }
    written
}

#[no_mangle]
pub fn stdin_read_bytes(buffer: &mut [u8]) -> Result<usize, Error> {
    let result = match unsafe { SERIAL_PORT.as_mut() } {
        Some(port) => cortex_m::interrupt::free(|_| Ok(port.read(buffer).ok().unwrap_or(0))),
        None => Ok(0),
    };
    if RX_FULL.fetch_and(false, Ordering::Relaxed) {
        unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::OTG_FS) }
    }
    result
}

type Allocator = UsbBusAllocator<UsbBus<USB>>;

pub fn init(alloc: Allocator, board_name: &'static str) -> impl Fn() {
    let allocator: &'static mut Allocator = Box::leak(Box::new(alloc));
    let serial_port = SerialPort::new(allocator);
    unsafe { SERIAL_PORT = Some(serial_port) }
    let device = UsbDeviceBuilder::new(allocator, UsbVidPid(0x16c0, 0x27dd))
        .product(board_name)
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();
    unsafe { USB_DEVICE = MaybeUninit::new(device) }
    || unsafe { poll() }
}
