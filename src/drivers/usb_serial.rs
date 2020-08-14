use super::serial::Serial;
use usb_device::bus::{UsbBus, UsbBusAllocator};
use usb_device::prelude::*;
use usbd_serial::SerialPort;
use usbd_serial::UsbError;

type E = UsbError;

pub fn init<B: UsbBus>(alloc: &UsbBusAllocator<B>) -> (Serial<E, E, SerialPort<B>>, UsbDevice<B>) {
    let serial = SerialPort::new(alloc);
    let usb_device = UsbDeviceBuilder::new(alloc, UsbVidPid(0x16c0, 0x27dd))
        .product("rs-flight")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();
    return (Serial(serial), usb_device);
}
