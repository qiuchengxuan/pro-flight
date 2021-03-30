//! The root task.

use chips::stm32f4::{
    clock, dma,
    flash::{Flash, Sector},
    rtc,
    spi::BaudrateControl,
    systick, usb_serial,
};
use drivers::{led::LED, nvram::NVRAM};
use drone_core::fib::{new_fn, ThrFiberStreamPulse, Yielded};
use drone_cortexm::{reg::prelude::*, thr::prelude::*};
use drone_stm32_map::periph::{
    dma::{periph_dma2_ch0, periph_dma2_ch3},
    flash::periph_flash,
    rtc::periph_rtc,
    spi::periph_spi1,
    sys_tick::periph_sys_tick,
};
use futures::prelude::*;
use pro_flight::{
    components::{
        cli::{Command, CLI},
        flight_data::FlightDataHUB,
        imu::IMU,
        logger,
        positioning::Positioning,
        speedometer::Speedometer,
    },
    config,
    sync::DataWriter,
    sys::timer,
};
use stm32f4xx_hal::{
    gpio::{Edge, ExtiPin},
    otg_fs::{UsbBus, USB},
    prelude::*,
    stm32,
};

use crate::{
    flash::FlashWrapper, mpu6000::DmaSpiMPU6000, spi::Spi1, thread, thread::ThrsInit, Regs,
};

macro_rules! into_interrupt {
    ($syscfg:ident, $peripherals:ident, $gpio:expr) => {{
        let mut int = $gpio.into_pull_up_input();
        int.make_interrupt_source(&mut $syscfg);
        int.enable_interrupt(&mut $peripherals.EXTI);
        int.trigger_on_edge(&mut $peripherals.EXTI, Edge::FALLING);
        int
    }};
}

/// The root task handler.
#[inline(never)]
pub fn handler(reg: Regs, thr_init: ThrsInit) {
    let mut thread = thread::init(thr_init);
    thread.hard_fault.add_once(|| panic!("Hard Fault"));
    thread.rcc.enable_int();
    let rcc_cir = reg.rcc_cir.into_copy();

    reg.rcc_ahb1enr.modify(|r| r.set_dma2en());
    reg.rcc_apb1enr.pwren.set_bit();
    reg.rcc_apb2enr.modify(|r| r.set_spi1en());

    let regs = (reg.rcc_cfgr, reg.rcc_cr, reg.rcc_pllcfgr);
    clock::setup_pll(&mut thread.rcc, rcc_cir, regs, &reg.flash_acr).root_wait();
    systick::init(periph_sys_tick!(reg), thread.sys_tick);

    let mut peripherals = stm32::Peripherals::take().unwrap();
    let mut syscfg = peripherals.SYSCFG.constrain();
    let (gpio_a, gpio_b, gpio_c) =
        (peripherals.GPIOA.split(), peripherals.GPIOB.split(), peripherals.GPIOC.split());

    let mut led = LED::new(gpio_b.pb5.into_push_pull_output(), timer::SysTimer::new());

    reg.pwr_cr.modify(|r| r.set_dbp());
    reg.rcc_bdcr.modify(|r| r.set_rtcsel1().set_rtcsel0().set_rtcen()); // select HSE
    rtc::init(periph_rtc!(reg));
    logger::init(Box::leak(Box::new([0u8; 1024])));

    let (usb_global, usb_device, usb_pwrclk) =
        (peripherals.OTG_FS_GLOBAL, peripherals.OTG_FS_DEVICE, peripherals.OTG_FS_PWRCLK);
    let (pin_dm, pin_dp) = (gpio_a.pa11.into_alternate_af10(), gpio_a.pa12.into_alternate_af10());
    let usb = USB { usb_global, usb_device, usb_pwrclk, pin_dm, pin_dp, hclk: clock::HCLK.into() };
    let allocator = UsbBus::new(usb, Box::leak(Box::new([0u32; 1024])));
    let mut poller = usb_serial::init(allocator);

    let flash = FlashWrapper::new(Flash::new(periph_flash!(reg)));
    let sector1 = unsafe { Sector::new(1).unwrap().as_slice() };
    let sector2 = unsafe { Sector::new(2).unwrap().as_slice() };
    let mut nvram = NVRAM::new(flash, [sector1, sector2]).unwrap();
    match nvram.load() {
        Ok(Some(config)) => config::replace(config),
        Ok(None) => (),
        Err(error) => error!("Load config failed: {:?}", error),
    }

    let hub = Box::leak(Box::new(FlightDataHUB::default()));
    let mut reader = hub.reader();

    let (heading, course) = (reader.gnss_heading, reader.gnss_course);
    let mut imu = IMU::new(reader.magnetometer, heading, course, 1000, 1000 / 10);
    let mut speedometer = Speedometer::new(reader.vertical_speed, reader.gnss_velocity, 1000, 10);
    let mut positioning = Positioning::new(reader.altimeter, reader.gnss_position, 1000);

    let pins = (gpio_a.pa5, gpio_a.pa6, gpio_a.pa7);
    let baudrate = BaudrateControl::new(clock::PCLK2, 1000u32.pow(2));
    let mpu6000 = DmaSpiMPU6000 {
        spi: Spi1::new(periph_spi1!(reg), pins, thread.spi_1, baudrate, mpu6000::SPI_MODE),
        cs: gpio_a.pa4.into_push_pull_output(),
        int: into_interrupt!(syscfg, peripherals, gpio_c.pc4),
        rx: dma::Channel::new(periph_dma2_ch0!(reg), thread.dma_2_stream_0),
        tx: dma::Channel::new(periph_dma2_ch3!(reg), thread.dma_2_stream_3),
        thread: thread.exti_4,
    };
    let (accelerometer, gyroscope) = (&hub.accelerometer, &hub.gyroscope);
    let (quat, speed) = (&hub.imu, &hub.speedometer);
    let (position, displacement) = (&hub.positioning, &hub.displacement);
    mpu6000.init(move |accel, gyro| {
        accelerometer.write(accel);
        gyroscope.write(gyro);
        if imu.update_imu(&accel, &gyro) {
            quat.write(imu.quaternion());
            let v = speedometer.update(imu.acceleration());
            speed.write(v);
            let (p, d) = positioning.update(v);
            position.write(p);
            displacement.write(d)
        }
    });
    let mut commands = [
        Command::new("reboot", "Reboot", |_| cortex_m::peripheral::SCB::sys_reset()),
        Command::new("logread", "Show log", |_| println!("{}", logger::get())),
        Command::new("telemetry", "Show flight data", move |_| println!("{}", reader.read())),
        Command::new("save", "Save configuration", move |_| {
            if let Some(err) = nvram.store(config::get()).err() {
                println!("Save configuration failed: {:?}", err);
                nvram.reset().ok();
            }
        }),
    ];
    let mut cli = CLI::new(&mut commands);
    let mut stream = thread.sys_tick.add_saturating_pulse_stream(new_fn(move || Yielded(Some(1))));
    while let Some(_) = stream.next().root_wait() {
        poller.poll(|bytes| cli.receive(bytes));
        led.check_toggle();
    }

    reg.scb_scr.sleeponexit.set_bit(); // Enter a sleep state on ISR exit.
}
