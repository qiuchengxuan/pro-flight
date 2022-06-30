//! The root task.

use alloc::boxed::Box;

use chips::stm32f4::{
    adc::IntoDMA as _,
    clock,
    delay::TickDelay,
    dfu, dma,
    flash::{Flash, Sector},
    rtc,
    softint::into_thread,
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
    flash::periph_flash,
    rtc::periph_rtc,
    spi::{periph_spi1, periph_spi3},
    sys_tick::periph_sys_tick,
};
use fugit::NanosDurationU64 as Duration;
use hal::{
    dma::DMA,
    persist::PersistDatastore,
    thread::{Schedule, Thread},
};
use pro_flight::{
    cli::CLI,
    config::{
        self,
        peripherals::serial::{Config as SerialConfig, RemoteControl as RC},
    },
    datastore,
    fcs::FCS,
    imu::IMU,
    ins,
    ins::variometer::Variometer,
    logger,
    protocol::serial,
    servo::pwm::PWMs,
    sync::event,
    sys::time::{self, TickTimer},
    sysinfo::{RebootReason, SystemInfo},
};
use stm32f4xx_hal::{
    adc::Adc,
    gpio::{Edge, ExtiPin},
    otg_fs::{UsbBus, USB},
    pac,
    prelude::*,
    serial::Serial,
    watchdog::IndependentWatchdog,
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
        int.trigger_on_edge(&mut $peripherals.EXTI, Edge::Falling);
        int
    }};
}

fn fiber_yield(mut f: impl FnMut()) -> impl FnMut() -> FiberState<(), ()> {
    move || {
        f();
        Yielded::<(), ()>(())
    }
}

/// The root task handler.
#[inline(never)]
pub fn handler(reg: Regs, thr_init: ThrsInit) {
    let mut threads = thread::init(thr_init);
    threads.hard_fault.add_once(|| panic!("Hard Fault"));
    let mut peripherals = pac::Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(168.mhz()).require_pll48clk().freeze();
    systick::init(periph_sys_tick!(reg), threads.sys_tick);
    thread::setup_priority(&mut threads);

    let (usb_global, usb_device, usb_pwrclk) =
        (peripherals.OTG_FS_GLOBAL, peripherals.OTG_FS_DEVICE, peripherals.OTG_FS_PWRCLK);
    let gpio_a = peripherals.GPIOA.split();
    let (pin_dm, pin_dp) = (gpio_a.pa11.into_alternate(), gpio_a.pa12.into_alternate());
    let usb = USB { usb_global, usb_device, usb_pwrclk, pin_dm, pin_dp, hclk: clocks.hclk() };
    static mut USB_BUFFER: [u32; 1024] = [0u32; 1024];
    let bus = UsbBus::new(usb, unsafe { &mut USB_BUFFER[..] });
    let poll = usb_serial::init(bus, board_name());
    threads.otg_fs.add_fn(move || {
        poll();
        Yielded::<(), ()>(())
    });
    threads.otg_fs.enable_int();

    logger::init(Box::leak(Box::new([0u8; 1024])));
    datastore::init();

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

    let pins = (gpio_a.pa5, gpio_a.pa6, gpio_a.pa7);
    let baudrate = BaudrateControl::new(clock::PCLK2, 1000u32.pow(2));
    let spi = Spi1::new(periph_spi1!(reg), pins, baudrate, mpu6000::SPI_MODE);
    let cs = gpio_a.pa4.into_push_pull_output();
    let mut mpu6000 = MPU6000::new(SpiBus::new(spi, cs, TickDelay {}));
    match mpu6000.init(SAMPLE_RATE as u16) {
        Ok(_) => info!("MPU6000 init OK"),
        Err(_) => error!("MPU6000 init failed"),
    }

    let mut ins = ins::INS::new(SAMPLE_RATE, Variometer::new(bmp280::SAMPLE_RATE));
    threads.ins.add_fn(fiber_yield(move || ins.update()));
    let mut ins_thread = into_thread(threads.ins);

    let rx = dma::Stream::new(periph_dma2_ch0!(reg), threads.dma2_stream0);
    let tx = dma::Stream::new(periph_dma2_ch3!(reg), threads.dma2_stream3);
    let mut imu = IMU::new(SAMPLE_RATE);
    let mpu6000 = mpu6000.into_dma((rx, 3), (tx, 3));
    let mut int = into_interrupt!(syscfg, peripherals, gpio_c.pc4);
    threads.mpu6000.add_fn(fiber_yield(move || int.clear_interrupt_pending_bit()));
    threads.mpu6000.add_exec(mpu6000.run(move |accel, gyro| {
        imu.update(accel.into(), gyro.into());
        ins_thread.wakeup()
    }));
    threads.mpu6000.enable_int();

    let dma_rx = dma::Stream::new(periph_dma2_ch2!(reg), threads.dma2_stream2);
    let mut adc = Adc::adc2(peripherals.ADC2, true, voltage_adc::adc_config());
    let vbat = gpio_c.pc2.into_analog();
    adc.configure_channel(&vbat, voltage_adc::SEQUENCE, voltage_adc::SAMPLE_TIME);
    adc.start_conversion();
    voltage_adc::init(adc.into_dma(), dma_rx, move |voltage| {
        datastore::acquire().write_voltage(voltage)
    });

    let pins = (gpio_c.pc10, gpio_c.pc11, gpio_c.pc12);
    let baudrate = BaudrateControl::new(clock::PCLK1, 10 * 1000u32.pow(2));
    let mut spi3 = Spi3::new(periph_spi3!(reg), pins, baudrate, bmp280::SPI_MODE);
    let mut rx = dma::Stream::new(periph_dma1_ch0!(reg), threads.dma1_stream0);
    rx.setup_peripheral(0, &mut spi3);
    let mut tx = dma::Stream::new(periph_dma1_ch5!(reg), threads.dma1_stream5);
    tx.setup_peripheral(0, &mut spi3);
    let cs_baro = gpio_b.pb3.into_push_pull_output();
    let mut bmp280 = bmp280_spi(spi3, cs_baro, TickDelay {});
    bmp280.init().map_err(|e| error!("Init bmp280 err: {:?}", e)).ok();
    let compensator = Compensator(bmp280.read_calibration().unwrap_or_default());
    let (spi, cs_baro, _) = bmp280.free().free();
    let bmp280 = DmaBMP280::new(rx.clone(), tx.clone(), cs_baro, compensator);
    let ds = datastore::acquire();
    threads.bmp280.add_exec(bmp280.run(move |v| ds.write_baro_altitude(v.into())));
    let thread = into_thread(threads.bmp280);
    let mut bmp280 = Schedule::new(thread, TickTimer::default(), Duration::millis(1));

    let mut cs_osd = gpio_a.pa15.into_push_pull_output();
    cs_osd.set_high();
    let event = event::Event::default();
    let max7456 = max7456::init(spi, cs_osd).unwrap();
    let max7456 = max7456.into_dma(event.clone(), tx).unwrap();
    threads.max7456.add_exec(max7456.run());
    let thread = into_thread(threads.max7456);
    let standard = config::get().osd.standard;
    let mut max7456 = Schedule::new(thread, TickTimer::default(), standard.refresh_interval());

    if let Some(config) = config::get().peripherals.serials.get("USART1") {
        let pins = (gpio_a.pa9.into_alternate(), gpio_a.pa10.into_alternate());
        let serial_config = usart::to_serial_config(&config);
        let usart1 = Serial::new(peripherals.USART1, pins, serial_config, &clocks).unwrap();
        let dma_rx = dma::Stream::new(periph_dma2_ch5!(reg), threads.dma2_stream5);
        if let Some(receiver) = serial::make_receiver(config) {
            usart::init(usart1.into_dma(), dma_rx, 4, receiver);
        }
    }

    // TODO: USART3 or I2C-2

    if let Some(config) = config::get().peripherals.serials.get("USART6") {
        if let SerialConfig::RC(RC::SBUS(sbus_config)) = config {
            if sbus_config.rx_inverted {
                gpio_c.pc8.into_push_pull_output().set_high();
                trace!("USART6 rx inverted");
            }
        }
        let pins = (gpio_c.pc6.into_alternate(), gpio_c.pc7.into_alternate());
        let serial_config = usart::to_serial_config(&config);
        let usart6 = Serial::new(peripherals.USART6, pins, serial_config, &clocks).unwrap();
        let dma_rx = dma::Stream::new(periph_dma2_ch1!(reg), threads.dma2_stream1);
        if let Some(receiver) = serial::make_receiver(config) {
            usart::init(usart6.into_dma(), dma_rx, 5, receiver);
        }
    }

    info!("Initialize PWMs");
    let tims = (peripherals.TIM1, peripherals.TIM2, peripherals.TIM3, peripherals.TIM5);
    let pins = (gpio_b.pb0, gpio_b.pb1, gpio_a.pa2, gpio_a.pa3, gpio_a.pa1, gpio_a.pa8);
    let pwms = crate::pwm::init(tims, pins, &clocks, &config::get().peripherals.pwms);
    let mut servos = PWMs::new(pwms);
    let mut fcs = FCS::new(SAMPLE_RATE);
    threads.fcs.add_fn(fiber_yield(move || {
        fcs.update();
        servos.update();
    }));

    let thread = into_thread(threads.fcs);
    let mut servos = Schedule::new(thread, TickTimer::default(), Duration::millis(20));

    threads.sys_tick.add_fn(fiber_yield(move || {
        bmp280.wakeup();
        max7456.wakeup();
        servos.wakeup();
    }));

    let mut watchdog = IndependentWatchdog::new(peripherals.IWDG);
    watchdog.start(500.ms());

    let commands =
        commands!((bootloader, [persist]), (osd, [event]), (save, [nvram]), (telemetry, []));
    let mut cli = CLI::new(commands);
    loop {
        TickTimer::after(Duration::millis(1)).root_wait();
        led.wakeup();
        cli.run();
        watchdog.feed();
    }
}
