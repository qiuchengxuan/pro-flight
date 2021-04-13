//! The root task.

use core::time::Duration;

use chips::stm32f4::{
    clock,
    delay::TickDelay,
    dfu, dma,
    flash::{Flash, Sector},
    rtc,
    softint::make_soft_int,
    spi::BaudrateControl,
    systick, usb_serial,
};
use drivers::{
    barometer::bmp280::{self, bmp280_spi, BMP280Init, Compensator, DmaBMP280},
    led::LED,
    nvram::NVRAM,
    stm32::voltage_adc,
};
use drone_core::fib::{new_fn, ThrFiberStreamPulse, Yielded};
use drone_cortexm::{reg::prelude::*, thr::prelude::*};
use drone_stm32_map::periph::{
    dma::{periph_dma1_ch0, periph_dma1_ch5, periph_dma2_ch0, periph_dma2_ch2, periph_dma2_ch3},
    exti::periph_exti2,
    flash::periph_flash,
    rtc::periph_rtc,
    spi::{periph_spi1, periph_spi3},
    sys_tick::periph_sys_tick,
};
use futures::prelude::*;
use hal::{
    dma::DMA,
    event::{Notifier, TimedNotifier},
    persist::PersistDatastore,
};
use pro_flight::{
    components::{
        cli::CLI, flight_data::FlightDataHUB, imu::IMU, logger, positioning::Positioning,
        speedometer::Speedometer, variometer::Variometer,
    },
    config,
    sync::DataWriter,
    sys::{time, timer},
    sysinfo::{RebootReason, SystemInfo},
};
use stm32f4xx_hal::{
    gpio::{Edge, ExtiPin},
    otg_fs::{UsbBus, USB},
    prelude::*,
    stm32,
};

use crate::{
    board_name,
    mpu6000::DmaSpiMPU6000,
    spi::{Spi1, Spi3},
    thread,
    thread::ThrsInit,
    Regs,
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
    let regs = (reg.rcc_cfgr, reg.rcc_cr, reg.rcc_pllcfgr);
    clock::setup_pll(&mut thread.rcc, rcc_cir, regs, &reg.flash_acr).root_wait();

    let mut peripherals = stm32::Peripherals::take().unwrap();
    let (usb_global, usb_device, usb_pwrclk) =
        (peripherals.OTG_FS_GLOBAL, peripherals.OTG_FS_DEVICE, peripherals.OTG_FS_PWRCLK);
    let gpio_a = peripherals.GPIOA.split();
    let (pin_dm, pin_dp) = (gpio_a.pa11.into_alternate_af10(), gpio_a.pa12.into_alternate_af10());
    let usb = USB { usb_global, usb_device, usb_pwrclk, pin_dm, pin_dp, hclk: clock::HCLK.into() };
    let bus = UsbBus::new(usb, Box::leak(Box::new([0u32; 1024])));
    let mut poller = usb_serial::init(bus, board_name());
    thread.otg_fs.add_fib(new_fn(move || {
        poller.poll();
        Yielded::<(), ()>(())
    }));
    thread.otg_fs.enable_int();

    systick::init(periph_sys_tick!(reg), thread.sys_tick);
    logger::init(Box::leak(Box::new([0u8; 1024])));

    reg.rcc_ahb1enr.modify(|r| r.set_dma1en().set_dma2en());
    reg.rcc_apb1enr.modify(|r| r.set_pwren().set_spi3en());
    reg.rcc_apb2enr.modify(|r| r.set_spi1en().set_adc2en());

    reg.pwr_cr.modify(|r| r.set_dbp());
    reg.rcc_bdcr.modify(|r| r.set_rtcsel1().set_rtcsel0().set_rtcen()); // select HSE
    let (rtc, mut persist) = rtc::init(periph_rtc!(reg));
    let mut sysinfo: SystemInfo = persist.load();
    match sysinfo.reboot_reason {
        RebootReason::Bootloader => {
            sysinfo.reboot_reason = RebootReason::Normal;
            persist.save(&sysinfo);
            dfu::enter();
        }
        _ => (),
    };

    let mut syscfg = peripherals.SYSCFG.constrain();
    let (gpio_b, gpio_c) = (peripherals.GPIOB.split(), peripherals.GPIOC.split());

    let mut led = LED::new(gpio_b.pb5.into_push_pull_output(), timer::SysTimer::new());

    let reader = rtc.reader();
    time::init(reader, rtc);

    let flash = Flash::new(periph_flash!(reg));
    let sector1 = unsafe { Sector::new(1).unwrap().as_slice() };
    let sector2 = unsafe { Sector::new(2).unwrap().as_slice() };
    let mut nvram = NVRAM::new(flash, [sector1, sector2]);
    match nvram.init().and(nvram.load()) {
        Ok(option) => config::replace(option.unwrap_or_default()),
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
        rx: dma::Stream::new(periph_dma2_ch0!(reg), thread.dma_2_stream_0),
        tx: dma::Stream::new(periph_dma2_ch3!(reg), thread.dma_2_stream_3),
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

    let dma_rx = dma::Stream::new(periph_dma2_ch2!(reg), thread.dma_2_stream_2);
    let battery = &hub.battery;
    voltage_adc::init(peripherals.ADC2, gpio_c.pc2, dma_rx, move |voltage| battery.write(voltage));

    let pins = (gpio_c.pc10, gpio_c.pc11, gpio_c.pc12);
    let baudrate = BaudrateControl::new(clock::PCLK1, 10 * 1000u32.pow(2));
    let spi3 = Spi3::new(periph_spi3!(reg), pins, thread.spi_3, baudrate, bmp280::SPI_MODE);
    let cs_baro = gpio_b.pb3.into_push_pull_output();
    let mut cs_osd = gpio_a.pa15.into_push_pull_output();
    cs_osd.set_high().ok();
    let mut bmp280 = bmp280_spi(spi3, cs_baro, TickDelay {});
    bmp280.init().map_err(|e| error!("Init bmp280 err: {:?}", e)).ok();
    let compensator = Compensator(bmp280.read_calibration().unwrap_or_default());
    let (mut spi, cs, _) = bmp280.free().free();

    let (altimeter, vertical_speed) = (&hub.altimeter, &hub.vertical_speed);
    let mut vs = Variometer::new(1000 / bmp280::SAMPLE_RATE);
    let mut bmp280 = DmaBMP280::new(cs, compensator, move |v| {
        altimeter.write(v.into());
        vertical_speed.write(vs.update(v.into()));
    });

    let mut rx = dma::Stream::new(periph_dma1_ch0!(reg), thread.dma_1_stream_0);
    rx.setup_peripheral(0, &mut spi);
    let mut tx = dma::Stream::new(periph_dma1_ch5!(reg), thread.dma_1_stream_5);
    tx.setup_peripheral(0, &mut spi);
    let int = make_soft_int(thread.exti_2, periph_exti2!(reg), move |_cx| bmp280.trigger(&rx, &tx));
    let mut bmp280 = TimedNotifier::new(int, timer::SysTimer::new(), Duration::from_millis(100));

    let mut commands = commands!((bootloader, [persist]), (telemetry, [reader]), (save, [nvram]));
    let mut cli = CLI::new(&mut commands);
    let mut stream = thread.sys_tick.add_saturating_pulse_stream(new_fn(move || Yielded(Some(1))));
    while let Some(_) = stream.next().root_wait() {
        let mut buffer = [0u8; 80];
        cli.receive(usb_serial::read(&mut buffer[..]));
        led.notify();
        bmp280.notify();
    }

    reg.scb_scr.sleeponexit.set_bit(); // Enter a sleep state on ISR exit.
}
