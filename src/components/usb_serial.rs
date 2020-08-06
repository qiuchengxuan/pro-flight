use usb_device::class_prelude::{UsbBus, UsbBusAllocator};
use usb_device::prelude::*;
use usbd_serial::SerialPort;

pub fn init<B: UsbBus>(allocator: &UsbBusAllocator<B>) -> (SerialPort<B>, UsbDevice<B>) {
    let serial = SerialPort::new(allocator);
    let usb_device = UsbDeviceBuilder::new(allocator, UsbVidPid(0x16c0, 0x27dd))
        .product("rs-flight")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();
    return (serial, usb_device);
}
