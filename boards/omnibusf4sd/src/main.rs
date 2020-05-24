#![no_main]
#![no_std]

#[macro_use]
extern crate cortex_m_rt;
extern crate btoi;
extern crate cast;
extern crate cortex_m;
extern crate cortex_m_systick_countdown;
extern crate max7456;
extern crate nb;
extern crate panic_semihosting;
extern crate stm32f4xx_hal;
extern crate usb_device;
#[macro_use]
extern crate mpu6000;
extern crate chips;
extern crate rs_flight;

mod console;
mod spi1_exti4_gyro;
mod spi3_tim7_osd_baro;

mod usb_serial;

use core::mem::MaybeUninit;

use arrayvec::ArrayVec;
use btoi::btoi_radix;
use cortex_m_rt::ExceptionFrame;
use cortex_m_systick_countdown::{MillisCountDown, PollingSysTick, SysTickCalibration};
use numtoa::NumToA;
use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use stm32f4xx_hal::pwm;
use stm32f4xx_hal::{prelude::*, stm32};

use chips::stm32f4::dfu::Dfu;
use rs_flight::components::imu::{get_handler, imu};
use rs_flight::components::sysled::Sysled;
use rs_flight::datastructures::event::event_nop_handler;
use rs_flight::hal::sensors::Temperature;

use console::Console;

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

#[entry]
fn main() -> ! {
    let mut dfu = Dfu::new();
    dfu.check();

    let cortex_m_peripherals = cortex_m::Peripherals::take().unwrap();
    let peripherals = stm32::Peripherals::take().unwrap();

    let rcc = peripherals.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(64.mhz())
        .require_pll48clk()
        .freeze();

    let mut delay = Delay::new(cortex_m_peripherals.SYST, clocks);

    let gpio_a = peripherals.GPIOA.split();
    let gpio_b = peripherals.GPIOB.split();
    let gpio_c = peripherals.GPIOC.split();

    // let pb0_1 = (
    //     gpio_b.pb0.into_alternate_af2(),
    //     gpio_b.pb1.into_alternate_af2(),
    // );
    // let (mut pwm1, mut pwm2) = pwm::tim3(peripherals.TIM3, pb0_1, clocks, 50.hz());

    // let pwm3_4 = (
    //     gpio_a.pa3.into_alternate_af1(),
    //     gpio_a.pa2.into_alternate_af1(),
    // );
    // let pwm2 = pwm::tim2(peripherals.TIM2, pwm3_4, clocks, 20u32.khz());

    // let pwm3 = pwm::tim5(
    //     peripherals.TIM5,
    //     gpio_a.pa1.into_alternate_af2(),
    //     clocks,
    //     20u32.khz(),
    // );

    // let pwm4 = pwm::tim1(
    //     peripherals.TIM1,
    //     gpio_a.pa8.into_alternate_af1(),
    //     clocks,
    //     20u32.khz(),
    // );

    let cs = gpio_a.pa4.into_push_pull_output();
    let sclk = gpio_a.pa5.into_alternate_af5();
    let miso = gpio_a.pa6.into_alternate_af5();
    let mosi = gpio_a.pa7.into_alternate_af5();
    let pins = (sclk, miso, mosi);
    let handlers = (get_handler(), event_nop_handler as fn(_: Temperature<u16>));
    let result = spi1_exti4_gyro::init(peripherals.SPI1, pins, cs, clocks, handlers, &mut delay);
    result.ok();

    let _cs = gpio_a.pa15.into_push_pull_output();
    let sclk = gpio_c.pc10.into_alternate_af6();
    let miso = gpio_c.pc11.into_alternate_af6();
    let mosi = gpio_c.pc12.into_alternate_af6();
    spi3_tim7_osd_baro::init(
        peripherals.SPI3,
        peripherals.TIM7,
        (sclk, miso, mosi),
        clocks,
        imu(),
        &mut delay,
    )
    .ok();

    let calibration = SysTickCalibration::from_clock_hz(clocks.sysclk().0);
    let systick = PollingSysTick::new(delay.free(), &calibration);

    let pin = gpio_b.pb5.into_push_pull_output();
    let mut sysled = Sysled::new(pin, MillisCountDown::new(&systick));

    let usb = USB {
        usb_global: peripherals.OTG_FS_GLOBAL,
        usb_device: peripherals.OTG_FS_DEVICE,
        usb_pwrclk: peripherals.OTG_FS_PWRCLK,
        pin_dm: gpio_a.pa11.into_alternate_af10(),
        pin_dp: gpio_a.pa12.into_alternate_af10(),
    };

    let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });
    let mut usb_serial = usb_serial::USBSerial::new(&usb_bus);
    let console = Console::new(&mut usb_serial);

    let mut vec = ArrayVec::<[u8; 80]>::new();
    loop {
        sysled.check_toggle().unwrap();

        let option = console.try_read_line(&mut vec);
        if option.is_none() {
            continue;
        }
        let line = option.unwrap();
        if line.len() > 0 {
            if line == *b"dfu" {
                dfu.reboot_into();
            } else if line == *b"reboot" {
                cortex_m::peripheral::SCB::sys_reset();
            } else if line.starts_with(b"read") {
                let mut iter = line.split(|b| *b == ' ' as u8);
                iter.next();
                let address = if let Some(address) = iter.next() {
                    match btoi_radix::<u32>(address, 16) {
                        Ok(address) => address,
                        _ => 0,
                    }
                } else {
                    0
                };
                if 0x40000000 <= address && address <= 0xA0000FFF {
                    let value = unsafe { *(address as *const u32) };
                    let mut buffer = [0u8; 10];
                    console.write(b"Result: ");
                    console.write(value.numtoa(16, &mut buffer));
                    console.write(b"\r\n")
                }
            } else if line.starts_with(b"dump") {
                let mut iter = line.split(|b| *b == ' ' as u8);
                iter.next();
                let mut address = if let Some(address) = iter.next() {
                    match btoi_radix::<u32>(address, 16) {
                        Ok(address) => address,
                        _ => 0,
                    }
                } else {
                    0
                };
                let size = if let Some(value) = iter.next() {
                    match btoi_radix::<u32>(value, 16) {
                        Ok(value) => value,
                        _ => {
                            address = 0;
                            0
                        }
                    }
                } else {
                    address = 0;
                    0
                };
                if 0x40000000 <= address && address <= 0xA0000FFF {
                    console.write(b"Dump result: ");
                    let mut buffer = [0u8; 10];
                    for i in 0..size {
                        let value = unsafe { *((address + i) as *const u32) };
                        console.write(value.numtoa(16, &mut buffer));
                        console.write(b" ");
                    }
                    console.write(b"\r\n");
                } else {
                    console.write(b"Bad input\r\n");
                }
            } else if line.starts_with(b"write") {
                let mut iter = line.split(|b| *b == ' ' as u8);
                iter.next();
                let mut address = if let Some(address) = iter.next() {
                    match btoi_radix::<u32>(address, 16) {
                        Ok(address) => address,
                        _ => 0,
                    }
                } else {
                    0
                };
                let value = if let Some(value) = iter.next() {
                    match btoi_radix::<u32>(value, 16) {
                        Ok(value) => value,
                        _ => {
                            address = 0;
                            0
                        }
                    }
                } else {
                    address = 0;
                    0
                };
                if 0x40000000 <= address && address <= 0xA0000FFF {
                    unsafe { *(address as *mut u32) = value };
                    let mut count_down = MillisCountDown::new(&systick);
                    count_down.start_ms(50);
                    nb::block!(count_down.wait()).unwrap();
                    let value = unsafe { *(address as *const u32) };
                    console.write(b"Write result: ");
                    let mut buffer = [0u8; 10];
                    console.write(value.numtoa(16, &mut buffer));
                    console.write(b"\r\n");
                } else {
                    console.write(b"Bad input\r\n");
                }
            } else {
                console.write(b"unknown input: ");
                console.write(line);
                console.write(b"\r\n")
            }
        }
        console.write(b"# ");
        vec.clear();
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
