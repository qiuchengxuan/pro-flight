use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;

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

    pub fn poll(&mut self) -> bool {
        self.dev.poll(&mut [&mut self.port])
    }

    pub fn read<'b>(&mut self, buffer: &'b mut [u8]) -> &'b [u8] {
        let size = match self.port.read(buffer) {
            Ok(size) => size,
            _ => 0,
        };
        return &buffer[..size];
    }

    pub fn write(&mut self, output: &[u8]) {
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
