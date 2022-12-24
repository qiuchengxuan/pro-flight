use drivers::stm32::usb_serial;
use drone_core::fib::{ThrFiberClosure, Yielded};
use drone_cortexm::thr::prelude::*;
use stm32f4xx_hal::{
    gpio::{
        gpioa::{PA11, PA12},
        Input,
    },
    otg_fs::{UsbBus, USB},
    pac,
    time::Hertz,
};

use crate::thread::OtgFs;

type OTGFSs = (pac::OTG_FS_GLOBAL, pac::OTG_FS_DEVICE, pac::OTG_FS_PWRCLK);

type PINs = (PA11<Input>, PA12<Input>);

pub fn init(otg_fs: OTGFSs, pins: PINs, hclk: Hertz, thread: OtgFs) {
    let (usb_global, usb_device, usb_pwrclk) = otg_fs;
    let (pin_dm, pin_dp) = (pins.0.into_alternate(), pins.1.into_alternate());
    let usb = USB { usb_global, usb_device, usb_pwrclk, pin_dm, pin_dp, hclk };
    static mut USB_BUFFER: [u32; 1024] = [0u32; 1024];
    let bus = UsbBus::new(usb, unsafe { &mut USB_BUFFER[..] });
    let poll = usb_serial::init(bus, crate::board_name());
    thread.add_fn(move || {
        poll();
        Yielded::<(), ()>(())
    });
    thread.enable_int();
}
