//! The root task.

use alloc::boxed::Box;
use core::time::Duration;

use chips::stm32f4::{
    adc::IntoDMA as _,
    clock,
    delay::TickDelay,
    dfu, dma,
    flash::{Flash, Sector},
    rtc,
    softint::make_trigger,
    spi::BaudrateControl,
    systick,
    usart::IntoDMA as _,
};
use drivers::{
    barometer::bmp280::{self, bmp280_spi, BMP280Init, Compensator, DmaBMP280},
    led::LED,
    max7456::{self, IntoDMA as _},
    mpu6000::{self, IntoDMA as _, MPU6000Init, SpiBus, MPU6000},
    nvram::NVRAM,
    stm32::{usart, usb_serial, voltage_adc},
};
use drone_core::fib::{FiberState, ThrFiberClosure, Yielded};
use drone_cortexm::{reg::prelude::*, thr::prelude::*};
use drone_stm32_map::periph::{
    dma::*,
    exti::{periph_exti1, periph_exti2, periph_exti3},
    flash::periph_flash,
    rtc::periph_rtc,
    spi::{periph_spi1, periph_spi3},
    sys_tick::periph_sys_tick,
};
use hal::{
    dma::DMA,
    event::{Notifier, TimedNotifier},
    persist::PersistDatastore,
};
use pro_flight::{
    components::{
        cli::CLI, flight_control::FlightControl, flight_data_hub::FlightDataHUB, logger,
        mixer::ControlMixer, pipeline, variometer::Variometer,
    },
    config::{self, peripherals::serial::Config as SerialConfig},
    protocol::serial,
    sync::{flag, DataWriter},
    sys::time::{self, TickTimer},
    sysinfo::{RebootReason, SystemInfo},
};
use stm32f4xx_hal::{
    adc::Adc,
    gpio::{Edge, ExtiPin},
    otg_fs::{UsbBus, USB},
    prelude::*,
    serial::Serial,
    stm32,
};

use crate::{
    board_name,
    spi::{Spi1, Spi3},
    thread,
    thread::ThrsInit,
    Regs,
};

const SAMPLE_RATE: usize = 1000;

macro_rules! into_interrupt {
    ($syscfg:ident, $peripherals:ident, $gpio:expr) => {{
        let mut int = $gpio.into_pull_up_input();
        int.make_interrupt_source(&mut $syscfg);
        int.enable_interrupt(&mut $peripherals.EXTI);
        int.trigger_on_edge(&mut $peripherals.EXTI, Edge::FALLING);
        int
    }};
}

fn never_complete(mut f: impl FnMut()) -> impl FnMut() -> FiberState<(), ()> {
    move || {
        f();
        Yielded::<(), ()>(())
    }
}

/// The root task handler.
#[inline(never)]
pub fn handler(reg: Regs, thr_init: ThrsInit) {
    let mut thread = thread::init(thr_init);
    thread.hard_fault.add_once(|| panic!("Hard Fault"));
    let mut peripherals = stm32::Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(168.mhz()).require_pll48clk().freeze();
    systick::init(periph_sys_tick!(reg), thread.sys_tick);
    thread::setup_priority(&mut thread);

    let (usb_global, usb_device, usb_pwrclk) =
        (peripherals.OTG_FS_GLOBAL, peripherals.OTG_FS_DEVICE, peripherals.OTG_FS_PWRCLK);
    let gpio_a = peripherals.GPIOA.split();
    let (pin_dm, pin_dp) = (gpio_a.pa11.into_alternate_af10(), gpio_a.pa12.into_alternate_af10());
    let usb = USB { usb_global, usb_device, usb_pwrclk, pin_dm, pin_dp, hclk: clocks.hclk() };
    static mut USB_BUFFER: [u32; 1024] = [0u32; 1024];
    let bus = UsbBus::new(usb, unsafe { &mut USB_BUFFER[..] });
    let poll = usb_serial::init(bus, board_name());
    thread.otg_fs.add_fn(move || {
        poll();
        Yielded::<(), ()>(())
    });
    thread.otg_fs.enable_int();

    logger::init(Box::leak(Box::new([0u8; 1024])));

    reg.rcc_ahb1enr.modify(|r| r.set_dma1en().set_dma2en().set_crcen());
    reg.rcc_apb1enr.modify(|r| r.set_pwren().set_spi3en());
    reg.rcc_apb2enr.modify(|r| r.set_spi1en().set_adc2en());

    let (rtc, mut persist) = rtc::init(periph_rtc!(reg), rtc::ClockSource::HSE);
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

    let mut led = LED::new(gpio_b.pb5.into_push_pull_output(), TickTimer::default());

    let reader = rtc.reader();
    time::init(reader, rtc);

    let flash = Flash::new(periph_flash!(reg));
    let sector1 = unsafe { Sector::new(1).unwrap().as_slice() };
    let sector2 = unsafe { Sector::new(2).unwrap().as_slice() };
    let mut nvram = NVRAM::new(flash, [sector1, sector2]);
    match nvram.init().and(nvram.load()) {
        Ok(Some(ref config)) => config::replace(&config),
        Ok(None) => config::reset(),
        Err(error) => error!("Load config failed: {:?}", error),
    }

    let hub: &'static FlightDataHUB = Box::leak(Box::new(FlightDataHUB::default()));
    let mut reader = hub.reader();

    let pins = (gpio_a.pa5, gpio_a.pa6, gpio_a.pa7);
    let baudrate = BaudrateControl::new(clock::PCLK2, 1000u32.pow(2));
    let spi = Spi1::new(periph_spi1!(reg), pins, baudrate, mpu6000::SPI_MODE);
    let cs = gpio_a.pa4.into_push_pull_output();
    let mut mpu6000 = MPU6000::new(SpiBus::new(spi, cs, TickDelay {}));
    match mpu6000.init(SAMPLE_RATE as u16) {
        Ok(_) => info!("MPU6000 init OK"),
        Err(_) => error!("MPU6000 init failed"),
    }

    let rx = dma::Stream::new(periph_dma2_ch0!(reg), thread.dma2_stream0);
    let tx = dma::Stream::new(periph_dma2_ch3!(reg), thread.dma2_stream3);
    let mut mpu6000 = mpu6000.into_dma((rx, 3), (tx, 3));
    let mut imu = pipeline::imu::IMU::new(SAMPLE_RATE, &hub);
    mpu6000.set_callback(move |accel, gyro| {
        hub.accelerometer.write(accel);
        hub.gyroscope.write(gyro);
        imu.invoke();
    });
    let mut int = into_interrupt!(syscfg, peripherals, gpio_c.pc4);
    thread.mpu6000.add_fn(never_complete(move || int.clear_interrupt_pending_bit()));
    thread.mpu6000.add_fn(never_complete(move || mpu6000.trigger()));
    thread.mpu6000.enable_int();

    let dma_rx = dma::Stream::new(periph_dma2_ch2!(reg), thread.dma2_stream2);
    let voltmeter = &hub.voltmeter;
    let mut adc = Adc::adc2(peripherals.ADC2, true, voltage_adc::adc_config());
    let vbat = gpio_c.pc2.into_analog();
    adc.configure_channel(&vbat, voltage_adc::SEQUENCE, voltage_adc::SAMPLE_TIME);
    adc.start_conversion();
    voltage_adc::init(adc.into_dma(), dma_rx, move |voltage| voltmeter.write(voltage));

    let pins = (gpio_c.pc10, gpio_c.pc11, gpio_c.pc12);
    let baudrate = BaudrateControl::new(clock::PCLK1, 10 * 1000u32.pow(2));
    let mut spi3 = Spi3::new(periph_spi3!(reg), pins, baudrate, bmp280::SPI_MODE);
    let mut rx = dma::Stream::new(periph_dma1_ch0!(reg), thread.dma1_stream0);
    rx.setup_peripheral(0, &mut spi3);
    let mut tx = dma::Stream::new(periph_dma1_ch5!(reg), thread.dma1_stream5);
    tx.setup_peripheral(0, &mut spi3);

    let mut cs_osd = gpio_a.pa15.into_push_pull_output();
    cs_osd.set_high().ok();
    let cs_baro = gpio_b.pb3.into_push_pull_output();
    let mut bmp280 = bmp280_spi(spi3, cs_baro, TickDelay {});
    bmp280.init().map_err(|e| error!("Init bmp280 err: {:?}", e)).ok();
    let compensator = Compensator(bmp280.read_calibration().unwrap_or_default());
    let (spi, cs_baro, _) = bmp280.free().free();

    let (altimeter, vertical_speed) = (&hub.altimeter, &hub.vertical_speed);
    let mut vs = Variometer::new(1000 / bmp280::SAMPLE_RATE);
    let mut bmp280 = DmaBMP280::new(cs_baro, compensator, move |v| {
        altimeter.write(v.into());
        vertical_speed.write(vs.update(v.into()));
    });

    let (setter, receiver) = flag();
    let max7456 = max7456::init(spi, cs_osd).unwrap();
    let max7456 = max7456.into_dma(receiver, tx.clone(), reader).unwrap();
    thread.max7456.add_exec(max7456.run());
    let int = make_trigger(thread.max7456, periph_exti3!(reg));
    let standard = config::get().osd.standard;
    let mut max7456 = TimedNotifier::new(int, TickTimer::default(), standard.refresh_interval());
    thread.bmp280.add_fn(never_complete(move || bmp280.trigger(&rx, &tx)));
    let int = make_trigger(thread.bmp280, periph_exti2!(reg));
    let mut bmp280 = TimedNotifier::new(int, TickTimer::default(), Duration::from_millis(100));

    if let Some(config) = config::get().peripherals.serials.get("USART1") {
        let pins = (gpio_a.pa9.into_alternate_af7(), gpio_a.pa10.into_alternate_af7());
        let serial_config = usart::to_serial_config(&config);
        let usart1 = Serial::usart1(peripherals.USART1, pins, serial_config, clocks).unwrap();
        let dma_rx = dma::Stream::new(periph_dma2_ch5!(reg), thread.dma2_stream5);
        if let Some(receiver) = serial::make_receiver(config, &hub) {
            usart::init(usart1.into_dma(), dma_rx, 4, receiver);
        }
    }

    // TODO: USART3 or I2C-2

    if let Some(config) = config::get().peripherals.serials.get("USART6") {
        if let SerialConfig::SBUS(sbus_config) = config {
            if sbus_config.rx_inverted {
                gpio_c.pc8.into_push_pull_output().set_high().ok();
                trace!("USART6 rx inverted");
            }
        }
        let pins = (gpio_c.pc6.into_alternate_af8(), gpio_c.pc7.into_alternate_af8());
        let serial_config = usart::to_serial_config(&config);
        let usart6 = Serial::usart6(peripherals.USART6, pins, serial_config, clocks).unwrap();
        let dma_rx = dma::Stream::new(periph_dma2_ch1!(reg), thread.dma2_stream1);
        if let Some(receiver) = serial::make_receiver(config, &hub) {
            usart::init(usart6.into_dma(), dma_rx, 5, receiver);
        }
    }

    info!("Initialize PWMs");
    let tims = (peripherals.TIM1, peripherals.TIM2, peripherals.TIM3, peripherals.TIM5);
    let pins = (gpio_b.pb0, gpio_b.pb1, gpio_a.pa2, gpio_a.pa3, gpio_a.pa1, gpio_a.pa8);
    let pwms = crate::pwm::init(tims, pins, clocks, &config::get().peripherals.pwms);
    let mixer = ControlMixer::new(reader.input, 50);
    let mut flight_control = FlightControl::new(mixer, &hub.output, pwms);
    thread.servo.add_fn(never_complete(move || flight_control.update()));

    let int = make_trigger(thread.servo, periph_exti1!(reg));
    let mut servos = TimedNotifier::new(int, TickTimer::default(), Duration::from_millis(20));

    thread.sys_tick.add_fn(never_complete(move || {
        max7456.notify();
        bmp280.notify();
        servos.notify();
    }));

    let commands =
        commands!((bootloader, [persist]), (osd, [setter]), (save, [nvram]), (telemetry, [reader]));
    let mut cli = CLI::new(commands);
    loop {
        TickTimer::after(Duration::from_millis(1)).root_wait();
        led.notify();
        cli.run();
    }
}
