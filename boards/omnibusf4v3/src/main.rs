#![no_main]
#![no_std]

#[macro_use]
extern crate ascii_osd_hud;
extern crate bmp280;
extern crate cast;
extern crate chips;
extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt;
extern crate cortex_m_systick_countdown;
extern crate embedded_sdmmc;
#[macro_use]
extern crate log;
extern crate max7456;
extern crate mpu6000;
extern crate nb;
#[macro_use]
extern crate rs_flight;
extern crate sbus_parser;
extern crate stm32f4xx_hal;
extern crate usb_device;

mod adc2_vbat;
mod pwm;
mod spi1_exti4_gyro;
mod spi2_exti7_sdcard;
mod spi3_tim7_osd_baro;
mod usart1;
mod usart6;
mod usb_serial;

use core::fmt::Write;
use core::mem::MaybeUninit;
use core::panic::PanicInfo;

use arrayvec::ArrayVec;
use chips::stm32f4::dfu::Dfu;
use chips::stm32f4::valid_memory_address;
use cortex_m_rt::ExceptionFrame;
use cortex_m_systick_countdown::{MillisCountDown, PollingSysTick, SysTickCalibration};
use log::Level;
use rs_flight::components::cmdlet;
use rs_flight::components::console::{self, Console};
use rs_flight::components::flight_control::{Aircraft, Airplane};
use rs_flight::components::logger::{self};
use rs_flight::components::panic::write_panic_file;
use rs_flight::components::{Altimeter, BatterySource, Sysled, TelemetryUnit, IMU};
use rs_flight::config::yaml::ToYAML;
use rs_flight::config::{read_config, Config, Output, SerialConfig};
use rs_flight::drivers::pwm::PwmByIdentifier;
use rs_flight::drivers::uart::Device;
use rs_flight::hal::input::{BasicInput, NoInput};
use rs_flight::sys::fs::File;
use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::gpio::{Edge, ExtiPin};
use stm32f4xx_hal::otg_fs::USB;
use stm32f4xx_hal::{prelude::*, stm32};

const MHZ: u32 = 1000_000;
const GYRO_SAMPLE_RATE: usize = 1000;
#[link_section = ".uninit.STACKS"]
#[link_section = ".ccmram"]
static mut LOG_BUFFER: [u8; 1024] = [0u8; 1024];
#[link_section = ".uninit.STACKS"]
static mut DFU: MaybeUninit<Dfu> = MaybeUninit::uninit();

static mut TELEMETRY: MaybeUninit<TelemetryUnit> = MaybeUninit::uninit();

#[entry]
fn main() -> ! {
    unsafe { &mut *DFU.as_mut_ptr() }.check();

    let cortex_m_peripherals = cortex_m::Peripherals::take().unwrap();
    let mut peripherals = stm32::Peripherals::take().unwrap();

    let rcc = peripherals.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(168.mhz()).freeze();

    unsafe { LOG_BUFFER = core::mem::zeroed() };
    logger::init(unsafe { &mut LOG_BUFFER }, Level::Trace);

    let (hclk, pclk1, pclk2) =
        (clocks.hclk().0 / MHZ, clocks.pclk1().0 / MHZ, clocks.pclk2().0 / MHZ);
    info!("hclk: {}mhz, pclk1: {}mhz, pclk2: {}mhz", hclk, pclk1, pclk2);
    info!("stack top: {:x}", cortex_m::register::msp::read());

    unsafe {
        let rcc = &*stm32::RCC::ptr();
        rcc.apb2enr.write(|w| w.syscfgen().enabled());
        rcc.ahb1enr.modify(|_, w| w.dma1en().enabled().dma2en().enabled());
    }

    let mut delay = Delay::new(cortex_m_peripherals.SYST, clocks);

    let gpio_a = peripherals.GPIOA.split();
    let gpio_b = peripherals.GPIOB.split();
    let gpio_c = peripherals.GPIOC.split();

    cmdlet::init(valid_memory_address);

    let mut int = gpio_b.pb7.into_pull_up_input();
    int.make_interrupt_source(&mut peripherals.SYSCFG);
    int.enable_interrupt(&mut peripherals.EXTI);
    int.trigger_on_edge(&mut peripherals.EXTI, Edge::RISING_FALLING);
    spi2_exti7_sdcard::init(
        peripherals.SPI2,
        (gpio_b.pb13, gpio_b.pb14, gpio_b.pb15),
        gpio_b.pb12,
        clocks,
        int,
    );

    info!("Loading config");
    let mut config: Config = Default::default();
    match File::open("sdcard://config.yml") {
        Ok(mut file) => {
            config = read_config(&mut file);
            file.close();
        }
        Err(e) => {
            warn!("{:?}", e);
        }
    };

    info!("Initialize PWMs");
    let pwms = (peripherals.TIM1, peripherals.TIM2, peripherals.TIM3, peripherals.TIM5);
    let pins = (gpio_b.pb0, gpio_b.pb1, gpio_a.pa2, gpio_a.pa3, gpio_a.pa1, gpio_a.pa8);
    let outputs = pwm::init(pwms, pins, clocks, &config.outputs);
    let mut airplane = Airplane::default();
    for &(identifier, output_type) in config.outputs.0.iter() {
        if let Some(output) = outputs.get(identifier) {
            match output_type {
                Output::AileronLeft => airplane.set_aileron_left(output),
                Output::AileronRight => airplane.set_aileron_right(output),
                Output::Elevator => airplane.set_elevator(output),
                Output::Rudder => airplane.set_rudder(output),
                Output::Motor(index, _) => airplane.set_motor(index as usize, output),
                Output::None => (),
            }
        }
    }

    let accel_gyro_ring = spi1_exti4_gyro::init_accel_gyro_ring();
    spi1_exti4_gyro::init_temperature_ring();
    let imu = IMU::new(accel_gyro_ring, GYRO_SAMPLE_RATE as u16, &config.accelerometer, 256);

    info!("Initialize MPU6000");
    let mut int = gpio_c.pc4.into_pull_up_input();
    int.make_interrupt_source(&mut peripherals.SYSCFG);
    int.enable_interrupt(&mut peripherals.EXTI);
    int.trigger_on_edge(&mut peripherals.EXTI, Edge::FALLING);
    spi1_exti4_gyro::init(
        peripherals.SPI1,
        (gpio_a.pa5, gpio_a.pa6, gpio_a.pa7),
        gpio_a.pa4,
        int,
        clocks,
        &mut delay,
        GYRO_SAMPLE_RATE as u16,
    )
    .ok();

    info!("Initialize ADC VBAT");
    let battery = BatterySource::new(adc2_vbat::init(peripherals.ADC2, gpio_c.pc2));

    let baro_ring = spi3_tim7_osd_baro::init_ring();
    let altimeter = Altimeter::new(baro_ring.clone());
    let telemetry = TelemetryUnit::new(imu, altimeter, battery, &config.battery);
    unsafe { TELEMETRY = MaybeUninit::new(telemetry) };
    let telemetry = unsafe { &*TELEMETRY.as_ptr() };

    info!("Initialize OSD & Barometer");
    spi3_tim7_osd_baro::init(
        peripherals.SPI3,
        peripherals.TIM7,
        (gpio_c.pc10, gpio_c.pc11, gpio_c.pc12),
        gpio_a.pa15,
        gpio_b.pb3,
        clocks,
        &config.osd,
        telemetry,
        &mut delay,
    )
    .ok();

    let calibration = SysTickCalibration::from_clock_hz(clocks.sysclk().0);
    let systick = PollingSysTick::new(delay.free(), &calibration);

    let pin = gpio_b.pb5.into_push_pull_output();
    let mut sysled = Sysled::new(pin, MillisCountDown::new(&systick));

    info!("Initialize USB CDC");
    let usb = USB {
        usb_global: peripherals.OTG_FS_GLOBAL,
        usb_device: peripherals.OTG_FS_DEVICE,
        usb_pwrclk: peripherals.OTG_FS_PWRCLK,
        pin_dm: gpio_a.pa11.into_alternate_af10(),
        pin_dp: gpio_a.pa12.into_alternate_af10(),
    };

    let (mut serial, mut device) = usb_serial::init(usb);

    let mut basic_input: &dyn BasicInput = &NoInput {};

    info!("Initialize USART1");
    if let Some(config) = config.serials.get("USART1") {
        if let SerialConfig::GNSS(baudrate) = config {
            let pins = (gpio_a.pa9, gpio_a.pa10);
            let count_down = MillisCountDown::new(&systick);
            usart1::init(peripherals.USART1, pins, baudrate, clocks, count_down);
        }
    }

    info!("Initialize USART6");
    if let Some(serial_config) = config.serials.get("USART6") {
        if let SerialConfig::SBUS(sbus_config) = serial_config {
            if sbus_config.rx_inverted {
                gpio_c.pc8.into_push_pull_output().set_high().ok();
                debug!("USART6 rx inverted");
            }
        }
        let pins = (gpio_c.pc6, gpio_c.pc7);
        let count_down = MillisCountDown::new(&systick);
        let device = usart6::init(peripherals.USART6, pins, &serial_config, clocks, count_down);
        match device {
            Device::SBUS(ref mut r) => {
                r.set_mapping(&config.receiver.0);
                basic_input = r;
            }
            _ => (),
        }
    }

    let mut aircraft = Aircraft::new(airplane, basic_input);

    let mut vec = ArrayVec::<[u8; 80]>::new();
    loop {
        aircraft.run_once();
        sysled.check_toggle().unwrap();
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
                unsafe { &mut *DFU.as_mut_ptr() }.reboot_into();
            } else if line.starts_with(b"reboot") {
                cortex_m::peripheral::SCB::sys_reset();
            } else if line.starts_with(b"logread") {
                for s in logger::reader() {
                    console::write(&mut serial, s).ok();
                }
            } else if line == *b"telemetry" {
                console!(&mut serial, "{}\n", unsafe { &*TELEMETRY.as_ptr() }.get_data());
            } else if line.starts_with(b"read") {
                cmdlet::read(line, &mut serial);
            } else if line.starts_with(b"dump ") {
                cmdlet::dump(line, &mut serial);
            } else if line.starts_with(b"write ") {
                let mut count_down = MillisCountDown::new(&systick);
                cmdlet::write(line, &mut serial, &mut count_down);
            } else if line.starts_with(b"show config") {
                config.write_to(0, &mut Console(&mut serial)).ok();
            } else {
                console!(&mut serial, "unknown input\n");
            }
        }
        console!(&mut serial, "# ");
        vec.clear();
    }
}

#[panic_handler]
unsafe fn panic(info: &PanicInfo) -> ! {
    write_panic_file(info);
    (&mut *DFU.as_mut_ptr()).reboot_into();
    loop {}
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    write_panic_file(ef);
    (&mut *DFU.as_mut_ptr()).reboot_into();
    loop {}
}

#[exception]
unsafe fn DefaultHandler(_irqn: i16) {
    (&mut *DFU.as_mut_ptr()).reboot_into();
}
