#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate ascii_osd_hud;
extern crate bmp280_core as bmp280;
extern crate cast;
extern crate chips;
extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt;
extern crate crc;
extern crate embedded_sdmmc;
extern crate max7456;
extern crate mpu6000;
extern crate nb;
#[macro_use]
extern crate rs_flight;
extern crate sbus_parser;
extern crate stm32f4xx_hal;

mod adc2_vbat;
mod exti0_softirq;
mod pwm;
mod spi1_exti4_gyro;
mod spi2_exti7_sdcard;
mod spi3_osd_baro;
mod tim7_scheduler;
mod usart1;
mod usart6;
mod watchdog;

use alloc::boxed::Box;
use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::panic::PanicInfo;
use core::time::Duration;

use alloc_cortex_m::CortexMHeap;
use chips::cortex_m4::{get_jiffies, systick_init};
use chips::stm32f4::crc::CRC;
use chips::stm32f4::dfu::Dfu;
use chips::stm32f4::valid_memory_address;
use cortex_m_rt::ExceptionFrame;
use rs_flight::components::altimeter::Altimeter;
use rs_flight::components::cli::memory;
use rs_flight::components::cli::CLI;
use rs_flight::components::configuration::Airplane;
use rs_flight::components::event::SchedulableEvent;
use rs_flight::components::imu::IMU;
use rs_flight::components::mixer::ControlMixer;
use rs_flight::components::navigation::Navigation;
use rs_flight::components::panic::{log_panic, PanicLogger};
use rs_flight::components::schedule::Scheduler;
use rs_flight::components::usb_serial;
use rs_flight::components::TelemetryUnit;
use rs_flight::config::aircraft::Configuration;
use rs_flight::config::{self, Config, SerialConfig};
use rs_flight::datastructures::data_source::{DataSource, NoDataSource};
use rs_flight::datastructures::input::ControlInput;
use rs_flight::drivers::bmp280::{init_data_source as init_bmp280_data_source, BMP280_SAMPLE_RATE};
use rs_flight::drivers::mpu6000::init_data_source as init_mpu6000_data_source;
use rs_flight::drivers::uart::Device;
use rs_flight::logger::{self, Level};
use rs_flight::sys::fs::File;
use rs_flight::sys::timer::{self, SysTimer};
use stm32f4xx_hal::gpio::{Edge, ExtiPin};
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::{prelude::*, stm32};

const MHZ: u32 = 1000_000;
const GYRO_SAMPLE_RATE: usize = 1000;
const LOG_BUFFER_SIZE: usize = 1024;

#[link_section = ".uninit.STACKS"]
#[link_section = ".ccmram"]
static mut CCM_MEMORY: [u8; 65536] = [0u8; 65536];

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

macro_rules! panic_logger {
    () => {
        &mut *(&mut CCM_MEMORY[LOG_BUFFER_SIZE] as *mut _ as *mut PanicLogger)
    };
}

#[link_section = ".uninit.STACKS"]
static mut DFU: MaybeUninit<Dfu> = MaybeUninit::uninit();

fn init(syst: cortex_m::peripheral::SYST, rcc: stm32::RCC) -> Clocks {
    let rcc = rcc.constrain();
    let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(168.mhz()).freeze();

    unsafe {
        let rcc = &*stm32::RCC::ptr();
        rcc.apb2enr.write(|w| w.syscfgen().enabled());
        rcc.ahb1enr.modify(|_, w| w.dma1en().enabled().dma2en().enabled().crcen().enabled());
    }

    systick_init(syst, clocks.sysclk().0);
    timer::init(get_jiffies);
    logger::init(Level::Debug);
    memory::init(valid_memory_address);

    let panic_logger = unsafe { panic_logger!() };
    if panic_logger.is_valid() {
        warn!("Last panic: {}", panic_logger);
        panic_logger.invalidate();
    }

    let sysclk = clocks.sysclk().0 / MHZ;
    let hclk = clocks.hclk().0 / MHZ;
    let pclk1 = clocks.pclk1().0 / MHZ;
    let pclk2 = clocks.pclk2().0 / MHZ;
    debug!("sysclk: {}mhz, hclk: {}mhz, pclk1: {}mhz, pclk2: {}mhz", sysclk, hclk, pclk1, pclk2);
    debug!("stack top: {:x}", cortex_m::register::msp::read());

    clocks
}

#[entry]
fn main() -> ! {
    let dfu = unsafe { &mut *DFU.as_mut_ptr() };
    dfu.check();
    dfu.arm();

    unsafe { ALLOCATOR.init(CCM_MEMORY.as_ptr() as usize, CCM_MEMORY.len()) }

    let cortex_m_peripherals = cortex_m::Peripherals::take().unwrap();
    let mut peripherals = stm32::Peripherals::take().unwrap();
    let mut nvic = cortex_m_peripherals.NVIC;

    let clocks = init(cortex_m_peripherals.SYST, peripherals.RCC);

    let gpio_a = peripherals.GPIOA.split();
    let gpio_b = peripherals.GPIOB.split();
    let gpio_c = peripherals.GPIOC.split();

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
            config = config::load(&mut file).clone();
            file.close();
        }
        Err(e) => {
            config::replace(config.clone());
            warn!("{:?}", e);
        }
    };

    let (accelerometer, gyroscope, _) = init_mpu6000_data_source();
    let rate = GYRO_SAMPLE_RATE as u16;
    let imu = IMU::new(accelerometer, gyroscope, rate, &config.accelerometer);

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
        GYRO_SAMPLE_RATE as u16,
    )
    .ok();

    info!("Initialize ADC VBAT");
    let battery = adc2_vbat::init(peripherals.ADC2, gpio_c.pc2).data_source();

    let barometer = init_bmp280_data_source();
    let altimeter = Altimeter::new(barometer, BMP280_SAMPLE_RATE as u16);

    let mut gnss: Option<&'static mut Device> = None;
    let mut receiver: Option<&'static mut Device> = None;

    if let Some(config) = config.serials.get("USART1") {
        info!("Initialize USART1");
        let pins = (gpio_a.pa9, gpio_a.pa10);
        if let Some(device) = usart1::init(peripherals.USART1, pins, &config, clocks) {
            match device {
                Device::GNSS(_) => gnss = Some(device),
                _ => (),
            }
        }
    }

    if let Some(serial_config) = config.serials.get("USART6") {
        info!("Initialize USART6");
        if let SerialConfig::SBUS(sbus_config) = serial_config {
            if sbus_config.rx_inverted {
                gpio_c.pc8.into_push_pull_output().set_high().ok();
                debug!("USART6 rx inverted");
            }
        }

        let pins = (gpio_c.pc6, gpio_c.pc7);
        let option = usart6::init(peripherals.USART6, pins, &mut nvic, &serial_config, clocks);
        if let Some(device) = option {
            match device {
                Device::SBUS(_) => receiver = Some(device),
                _ => (),
            }
        }
    }

    let mut led = gpio_b.pb5.into_push_pull_output();

    info!("Initialize USB CDC");
    let usb = USB {
        usb_global: peripherals.OTG_FS_GLOBAL,
        usb_device: peripherals.OTG_FS_DEVICE,
        usb_pwrclk: peripherals.OTG_FS_PWRCLK,
        pin_dm: gpio_a.pa11.into_alternate_af10(),
        pin_dp: gpio_a.pa12.into_alternate_af10(),
    };
    let allocator = UsbBus::new(usb, Box::leak(Box::new([0u32; 1024])));

    let (mut serial, mut device) = usb_serial::init(&allocator);

    let mut control_input: Box<dyn DataSource<ControlInput>> = Box::new(NoDataSource::new());
    if let Some(Device::SBUS(ref mut sbus)) = receiver {
        control_input = Box::new(sbus.as_control_input());
    }

    info!("Initialize PWMs");
    let tims = (peripherals.TIM1, peripherals.TIM2, peripherals.TIM3, peripherals.TIM5);
    let pins = (gpio_b.pb0, gpio_b.pb1, gpio_a.pa2, gpio_a.pa3, gpio_a.pa1, gpio_a.pa8);
    let pwms = pwm::init(tims, pins, clocks, &config.outputs);
    let mixer = ControlMixer::new(control_input, NoDataSource::new());
    let control_surface = match config.aircraft.configuration {
        Configuration::Airplane => Airplane::new(mixer, pwms),
    };

    let interval = 1.0 / GYRO_SAMPLE_RATE as f32;
    let mut navigation = Navigation::new(imu.as_imu(), imu.as_accelerometer(), interval);
    navigation.set_altimeter(Box::new(altimeter.as_data_source()));
    if let Some(Device::GNSS(ref mut gnss)) = gnss {
        navigation.set_gnss(Box::new(gnss.as_position_source()));
    }

    let mut telemetry = TelemetryUnit::new(
        altimeter.as_data_source(),
        battery,
        imu.as_accelerometer(),
        imu.as_imu(),
        navigation.as_data_source(),
    );
    if let Some(Device::SBUS(ref mut sbus)) = receiver {
        telemetry.set_receiver(Box::new(sbus.as_receiver()));
        telemetry.set_control_input(Box::new(sbus.as_control_input()));
    }

    info!("Initialize OSD & Barometer");
    let result = spi3_osd_baro::init(
        peripherals.SPI3,
        (gpio_c.pc10, gpio_c.pc11, gpio_c.pc12),
        (gpio_a.pa15, gpio_b.pb3),
        &mut CRC(peripherals.CRC),
        clocks,
        telemetry.as_data_source(),
    );
    let (baro, osd) = result.ok().unwrap();

    let trigger = exti0_softirq::init(&mut peripherals.EXTI, Box::new(control_surface));
    if let Some(Device::SBUS(ref mut sbus)) = receiver {
        sbus.set_notify(Box::new(trigger.clone()));
    }

    let mut watchdog = watchdog::init(peripherals.IWDG);

    let telemetry_source = telemetry.as_data_source();
    let schedule_trigger = SchedulableEvent::new(trigger, 50);
    let group =
        Scheduler::new((schedule_trigger, baro, altimeter, imu, navigation, telemetry, osd), 100);
    tim7_scheduler::init(peripherals.TIM7, Box::new(group), clocks, 100);

    let mut cli = CLI::new();
    let mut timer = SysTimer::new();
    loop {
        if timer.wait().is_ok() {
            timer.start(Duration::from_millis(100));
            watchdog.feed();
            led.toggle().ok();
        }

        if !device.poll(&mut [&mut serial]) {
            continue;
        }
        cli.interact(&mut serial, |line, serial| -> bool {
            match line.split(' ').next() {
                Some("dfu") => {
                    cortex_m::interrupt::disable();
                    cortex_m::peripheral::SCB::sys_reset();
                }
                Some("reboot") => {
                    dfu.disarm();
                    cortex_m::interrupt::disable();
                    cortex_m::peripheral::SCB::sys_reset()
                }
                Some("free") => {
                    console!(serial, "Used: {}, free: {}\n", ALLOCATOR.used(), ALLOCATOR.free());
                }
                Some("telemetry") => {
                    console!(serial, "{}\n", telemetry_source.read_last_unchecked())
                }
                _ => return false,
            }
            true
        });
    }
}

#[panic_handler]
unsafe fn panic(panic_info: &PanicInfo) -> ! {
    log_panic(format_args!("{}", panic_info), panic_logger!());
    cortex_m::peripheral::SCB::sys_reset();
}

#[exception]
unsafe fn HardFault(exception_frame: &ExceptionFrame) -> ! {
    log_panic(format_args!("{:?}", exception_frame), panic_logger!());
    cortex_m::peripheral::SCB::sys_reset();
}

#[exception]
unsafe fn DefaultHandler(irqn: i16) {
    log_panic(format_args!("{}", irqn), panic_logger!());
    cortex_m::peripheral::SCB::sys_reset();
}

#[alloc_error_handler]
unsafe fn oom(_: Layout) -> ! {
    log_panic(format_args!("OOM"), panic_logger!());
    cortex_m::peripheral::SCB::sys_reset();
}
