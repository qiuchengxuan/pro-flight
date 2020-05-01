#![no_main]
#![no_std]

#[macro_use]
extern crate cortex_m_rt;
extern crate cast;
extern crate cortex_m;
extern crate panic_semihosting;
extern crate stm32f4xx_hal;
extern crate usb_device;

use core::sync::atomic::{AtomicBool, Ordering};

use cortex_m_rt::ExceptionFrame;
use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use stm32f4xx_hal::timer::{Event, Timer};
use stm32f4xx_hal::{prelude::*, stm32};
use usb_device::prelude::*;

static mut G_TIM4: Option<Timer<stm32::TIM4>> = None;
static G_LED_ON: AtomicBool = AtomicBool::new(false);

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

#[interrupt]
fn TIM4() {
    cortex_m::interrupt::free(|_cs| unsafe {
        if let Some(ref mut tim) = G_TIM4 {
            tim.clear_interrupt(Event::TimeOut);
        };
    });
    let led_on = G_LED_ON.load(Ordering::Relaxed);
    G_LED_ON.store(!led_on, Ordering::Relaxed);
}

#[allow(unused_must_use)]
#[entry]
fn main() -> ! {
    let cortexm_peripherals = cortex_m::Peripherals::take().unwrap();
    let peripherals = stm32::Peripherals::take().unwrap();

    let rcc = peripherals.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(168.mhz())
        .require_pll48clk()
        .freeze();

    let gpio_b = peripherals.GPIOB.split();
    let mut led = gpio_b.pb5.into_push_pull_output();
    led.set_low();

    let mut timer = Timer::tim4(peripherals.TIM4, 10.hz(), clocks);
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::TIM4);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::TIM4) }
    timer.listen(Event::TimeOut);

    cortex_m::interrupt::free(|_cs| unsafe { G_TIM4 = Some(timer) });

    let gpio_a = peripherals.GPIOA.split();
    let usb = USB {
        usb_global: peripherals.OTG_FS_GLOBAL,
        usb_device: peripherals.OTG_FS_DEVICE,
        usb_pwrclk: peripherals.OTG_FS_PWRCLK,
        pin_dm: gpio_a.pa11.into_alternate_af10(),
        pin_dp: gpio_a.pa12.into_alternate_af10(),
    };

    let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });

    let mut serial = usbd_serial::SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .product("ng-plane")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

    let mut delay = Delay::new(cortexm_peripherals.SYST, clocks);
    loop {
        let led_on = G_LED_ON.load(Ordering::Relaxed);
        if led_on {
            led.set_low();
            delay.delay_ms(20_u16);
            led.set_high();
            delay.delay_ms(20_u16);
            led.set_low();
        } else {
            led.set_high();
        }

        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        let mut buf = [0u8; 64];

        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                // Echo back in upper case
                for c in buf[0..count].iter_mut() {
                    if 0x61 <= *c && *c <= 0x7a {
                        *c &= !0x20;
                    }
                }

                let mut write_offset = 0;
                while write_offset < count {
                    match serial.write(&buf[write_offset..count]) {
                        Ok(len) if len > 0 => {
                            write_offset += len;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
