use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;

use crate::console::{Available, ReadSome, Writable};

pub struct USBSerial<'a> {
    port: usbd_serial::SerialPort<'a, UsbBus<USB>>,
    dev: UsbDevice<'a, UsbBus<USB>>,
}

impl<'a> USBSerial<'a> {
    pub fn new(usb_bus: &'a UsbBusAllocator<UsbBus<USB>>) -> Self {
        Self {
            port: usbd_serial::SerialPort::new(usb_bus),
            dev: UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
                .product("ng-plane")
                .device_class(usbd_serial::USB_CLASS_CDC)
                .build(),
        }
    }
}

impl<'a> ReadSome for USBSerial<'a> {
    fn read_some<'b>(&mut self, buffer: &'b mut [u8]) -> &'b [u8] {
        let size = match self.port.read(buffer) {
            Ok(size) => size,
            _ => 0,
        };
        return &buffer[..size];
    }
}

impl<'a> Writable for USBSerial<'a> {
    fn write(&mut self, output: &[u8]) {
        let mut offset = 0;
        while offset < output.len() {
            match self.port.write(&output[offset..]) {
                Ok(len) => {
                    offset += len;
                }
                _ => {}
            }
        }
    }
}

impl<'a> Available for USBSerial<'a> {
    fn available(&mut self) -> bool {
        self.dev.poll(&mut [&mut self.port])
    }
}
