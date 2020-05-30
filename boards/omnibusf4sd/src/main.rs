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
#[macro_use]
extern crate rs_flight;

// mod software_interrupt;
mod spi1_exti4_gyro;
mod spi3_tim7_osd_baro;
mod usb_serial;

use core::fmt::Write;

use arrayvec::ArrayVec;
use btoi::btoi_radix;
use cortex_m_rt::ExceptionFrame;
use cortex_m_systick_countdown::{MillisCountDown, PollingSysTick, SysTickCalibration};

use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::gpio::Edge;
use stm32f4xx_hal::gpio::ExtiPin;
use stm32f4xx_hal::otg_fs::USB;
use stm32f4xx_hal::pwm;
use stm32f4xx_hal::{prelude::*, stm32};

use chips::stm32f4::dfu::Dfu;
use chips::stm32f4::valid_memory_address;
use rs_flight::components::console::{self, Console};
use rs_flight::components::imu::{self};
use rs_flight::components::logger::{self, Logger};
use rs_flight::components::sysled::Sysled;
use rs_flight::datastructures::event::event_nop_handler;
use rs_flight::hal::imu::IMU;
use rs_flight::hal::sensors::Temperature;

static mut LOG_BUFFER: [u8; 1024] = [0u8; 1024];

#[entry]
fn main() -> ! {
    let mut dfu = Dfu::new();
    dfu.check();

    let cortex_m_peripherals = cortex_m::Peripherals::take().unwrap();
    let mut peripherals = stm32::Peripherals::take().unwrap();

    let rcc = peripherals.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(168.mhz()).freeze();

    logger::init(unsafe { &mut LOG_BUFFER });
    log!("hclk: {}", clocks.hclk().0);

    unsafe {
        let rcc = &*stm32::RCC::ptr();
        rcc.apb2enr.write(|w| w.syscfgen().enabled());
    }

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
    let mut int = gpio_c.pc4.into_pull_up_input();
    int.make_interrupt_source(&mut peripherals.SYSCFG);
    int.enable_interrupt(&mut peripherals.EXTI);
    int.trigger_on_edge(&mut peripherals.EXTI, Edge::FALLING);
    let pins = (sclk, miso, mosi);
    let handlers = (imu::get_accel_gyro_handler(), event_nop_handler as fn(_: Temperature<i32>));
    spi1_exti4_gyro::init(peripherals.SPI1, pins, cs, int, clocks, handlers, &mut delay).ok();

    let imu = imu::init();

    let _cs = gpio_a.pa15.into_push_pull_output();
    let sclk = gpio_c.pc10.into_alternate_af6();
    let miso = gpio_c.pc11.into_alternate_af6();
    let mosi = gpio_c.pc12.into_alternate_af6();
    spi3_tim7_osd_baro::init(
        peripherals.SPI3,
        peripherals.TIM7,
        (sclk, miso, mosi),
        clocks,
        imu,
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

    let (mut serial, mut device) = usb_serial::init(usb);

    let mut vec = ArrayVec::<[u8; 80]>::new();
    loop {
        sysled.check_toggle().unwrap();
        // imu::trigger_handle();

        if !device.poll(&mut [&mut serial]) {
            continue;
        }

        let option = console::read_line(&mut serial, &mut vec);
        if option.is_none() {
            continue;
        }
        let line = option.unwrap();
        if line.len() > 0 {
            if line == *b"dfu" {
                dfu.reboot_into();
            } else if line == *b"reboot" {
                cortex_m::peripheral::SCB::sys_reset();
            } else if line == *b"logread" {
                for s in logger::reader() {
                    console::write(&mut serial, s).ok();
                }
            } else if line == *b"imu" {
                let attitude = imu.get_attitude();
                console!(
                    &mut serial,
                    "Pitch: {}, Roll: {}, Yaw: {}\r\n",
                    attitude.pitch,
                    attitude.roll,
                    attitude.yaw
                );
            } else if line.starts_with(b"read ") {
                if let Some(word) = line[5..].split(|b| *b == ' ' as u8).next() {
                    if let Some(address) = btoi_radix::<u32>(word, 16).ok() {
                        if valid_memory_address(address) {
                            let value = unsafe { *(address as *const u32) };
                            console!(&mut serial, "Result: {:x}\r\n", value);
                        }
                    }
                }
            } else if line.starts_with(b"readf ") {
                if let Some(word) = line[6..].split(|b| *b == ' ' as u8).next() {
                    if let Some(address) = btoi_radix::<u32>(word, 16).ok() {
                        if valid_memory_address(address) {
                            let value = unsafe { *(address as *const f32) };
                            console!(&mut serial, "Result: {}\r\n", value);
                        }
                    }
                }
            } else if line.starts_with(b"write ") {
                let mut iter = line[6..]
                    .split(|b| *b == ' ' as u8)
                    .flat_map(|w| btoi_radix::<u32>(w, 16).ok());
                if let Some(address) = iter.next() {
                    if let Some(value) = iter.next() {
                        if valid_memory_address(address) {
                            unsafe { *(address as *mut u32) = value };
                            let mut count_down = MillisCountDown::new(&systick);
                            count_down.start_ms(50);
                            nb::block!(count_down.wait()).unwrap();
                            let value = unsafe { *(address as *const u32) };
                            console!(&mut serial, "Write result: {:x}\r\n", value);
                        }
                    }
                }
            } else {
                console!(&mut serial, "unknown input\r\n");
            }
        }
        console!(&mut serial, "# ");
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
