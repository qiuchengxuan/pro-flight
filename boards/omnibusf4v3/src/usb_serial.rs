use core::mem::MaybeUninit;

use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::SerialPort;

use rs_flight::alloc;

pub fn init<'a>(usb: USB) -> (SerialPort<'a, UsbBus<USB>>, UsbDevice<'a, UsbBus<USB>>) {
    let ep_memory = alloc::typed_allocate(0u32, 1024, false);
    static mut USB_BUS: MaybeUninit<UsbBusAllocator<UsbBus<USB>>> = MaybeUninit::uninit();
    unsafe { USB_BUS = MaybeUninit::new(UsbBus::new(usb, ep_memory.unwrap())) };
    let serial = SerialPort::new(unsafe { &*USB_BUS.as_ptr() });
    let usb_device =
        UsbDeviceBuilder::new(unsafe { &*USB_BUS.as_ptr() }, UsbVidPid(0x16c0, 0x27dd))
            .product("rs-flight")
            .device_class(usbd_serial::USB_CLASS_CDC)
            .build();
    return (serial, usb_device);
}
