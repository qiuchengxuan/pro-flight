#![no_main]
#![no_std]

extern crate ascii_osd_hud;
extern crate bmp280_core as bmp280;
extern crate cast;
extern crate chips;
extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt;
extern crate cortex_m_systick_countdown;
extern crate embedded_sdmmc;
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

use core::mem::MaybeUninit;
use core::panic::PanicInfo;

use chips::stm32f4::dfu::Dfu;
use chips::stm32f4::valid_memory_address;
use cortex_m_rt::ExceptionFrame;
use cortex_m_systick_countdown::{MillisCountDown, PollingSysTick, SysTickCalibration};
use rs_flight::alloc;
use rs_flight::components::altimeter::Altimeter;
use rs_flight::components::cli::memory;
use rs_flight::components::cli::CLI;
use rs_flight::components::configuration::Airplane;
use rs_flight::components::imu::IMU;
use rs_flight::components::mixer::ControlMixer;
use rs_flight::components::navigation::Navigation;
use rs_flight::components::panic::{log_panic, PanicLogger};
use rs_flight::components::{Sysled, TelemetryUnit};
use rs_flight::config::aircraft::Configuration;
use rs_flight::config::{self, Config, SerialConfig};
use rs_flight::datastructures::data_source::{DataSource, NoDataSource};
use rs_flight::datastructures::input::ControlInput;
use rs_flight::datastructures::schedule::Schedulable;
use rs_flight::drivers::bmp280::{init_data_source as init_bmp280_data_source, BMP280_SAMPLE_RATE};
use rs_flight::drivers::mpu6000::init_data_source as init_mpu6000_data_source;
use rs_flight::drivers::uart::Device;
use rs_flight::logger::{self, Level};
use rs_flight::sys::fs::File;
use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::gpio::{Edge, ExtiPin};
use stm32f4xx_hal::otg_fs::USB;
use stm32f4xx_hal::{prelude::*, stm32};

const MHZ: u32 = 1000_000;
const GYRO_SAMPLE_RATE: usize = 1000;
const LOG_BUFFER_SIZE: usize = 1024;

#[link_section = ".uninit.STACKS"]
#[link_section = ".ccmram"]
static mut CCM_MEMORY: [u8; 32768] = [0u8; 32768];

macro_rules! panic_logger {
    () => {
        &mut *(&mut CCM_MEMORY[LOG_BUFFER_SIZE] as *mut _ as *mut PanicLogger)
    };
}

#[link_section = ".uninit.STACKS"]
static mut DFU: MaybeUninit<Dfu> = MaybeUninit::uninit();

#[entry]
fn main() -> ! {
    unsafe { &mut *DFU.as_mut_ptr() }.check();

    let cortex_m_peripherals = cortex_m::Peripherals::take().unwrap();
    let mut peripherals = stm32::Peripherals::take().unwrap();

    let rcc = peripherals.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(168.mhz()).freeze();

    unsafe { alloc::init(&mut [], &mut CCM_MEMORY) };

    logger::init(alloc::allocate(1024, false).unwrap(), Level::Debug);

    let panic_logger = unsafe { panic_logger!() };
    if panic_logger.is_valid() {
        warn!("Last panic: {}", panic_logger);
        panic_logger.invalidate();
    }

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

    memory::init(valid_memory_address);

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
            config = *config::load(&mut file);
            file.close();
        }
        Err(e) => {
            config::replace(&config);
            warn!("{:?}", e);
        }
    };

    let (accelerometer, gyroscope, _) = init_mpu6000_data_source();
    let rate = GYRO_SAMPLE_RATE as u16;
    let imu = IMU::new(accelerometer, gyroscope, rate, &config.accelerometer);
    let imu = alloc::into_static(imu, false).unwrap();

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
    let battery = adc2_vbat::init(peripherals.ADC2, gpio_c.pc2).data_source();

    let barometer = init_bmp280_data_source();
    let altimeter = Altimeter::new(barometer, BMP280_SAMPLE_RATE as u16);
    let altimeter = alloc::into_static(altimeter, false).unwrap();

    let interval = 1.0 / GYRO_SAMPLE_RATE as f32;
    let navigation = Navigation::new(imu.as_imu(), imu.as_accelerometer(), interval);
    let navigation = alloc::into_static(navigation, false).unwrap();
    navigation.set_altimeter(alloc::into_static(altimeter.as_data_source(), false).unwrap());

    if let Some(config) = config.serials.get("USART1") {
        info!("Initialize USART1");
        let pins = (gpio_a.pa9, gpio_a.pa10);
        match usart1::init(peripherals.USART1, pins, &config, clocks) {
            Device::GNSS(ref mut gnss) => {
                navigation.set_gnss(alloc::into_static(gnss.as_position_source(), false).unwrap());
            }
            _ => (),
        }
    }

    let mut telemetry = TelemetryUnit::new(
        altimeter.as_data_source(),
        battery,
        imu.as_accelerometer(),
        imu.as_imu(),
        navigation.as_data_source(),
    );

    let mut control_input: &'static mut dyn DataSource<ControlInput> =
        alloc::into_static(NoDataSource::new(), false).unwrap();

    if let Some(serial_config) = config.serials.get("USART6") {
        info!("Initialize USART6");
        if let SerialConfig::SBUS(sbus_config) = serial_config {
            if sbus_config.rx_inverted {
                gpio_c.pc8.into_push_pull_output().set_high().ok();
                debug!("USART6 rx inverted");
            }
        }

        let pins = (gpio_c.pc6, gpio_c.pc7);
        match usart6::init(peripherals.USART6, pins, &serial_config, clocks) {
            Device::SBUS(ref mut sbus) => {
                telemetry.set_receiver(alloc::into_static(sbus.as_receiver(), false).unwrap());
                let input = alloc::into_static(sbus.as_control_input(), false).unwrap();
                telemetry.set_control_input(input);
                control_input = alloc::into_static(sbus.as_control_input(), false).unwrap();
            }
            _ => (),
        }
    }

    let telemetry = alloc::into_static(telemetry, false).unwrap();

    info!("Initialize OSD & Barometer");
    spi3_tim7_osd_baro::init(
        peripherals.SPI3,
        peripherals.TIM7,
        (gpio_c.pc10, gpio_c.pc11, gpio_c.pc12),
        gpio_a.pa15,
        gpio_b.pb3,
        clocks,
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

    info!("Initialize PWMs");
    let tims = (peripherals.TIM1, peripherals.TIM2, peripherals.TIM3, peripherals.TIM5);
    let pins = (gpio_b.pb0, gpio_b.pb1, gpio_a.pa2, gpio_a.pa3, gpio_a.pa1, gpio_a.pa8);
    let pwms = pwm::init(tims, pins, clocks, &config.outputs);
    let mixer = ControlMixer::new(control_input, NoDataSource::new());
    let mut control_surface = match config.aircraft.configuration {
        Configuration::Airplane => Airplane::new(mixer, pwms),
    };

    let (primary, no_dma) = alloc::available();
    info!("Remain heap size: primary: {}, no-dma: {}", primary, no_dma);

    let mut cli = CLI::new(MillisCountDown::new(&systick));
    let mut schedule = MillisCountDown::new(&systick);
    schedule.start_ms(20);
    loop {
        if schedule.wait().is_ok() {
            altimeter.schedule();
            imu.schedule();
            navigation.schedule();
            control_surface.schedule();
            schedule.start_ms(20);
        }
        sysled.check_toggle().unwrap();
        if !device.poll(&mut [&mut serial]) {
            continue;
        }
        cli.interact(&mut serial, |line, serial| -> bool {
            match line.split(' ').next() {
                Some("dfu") => unsafe { &mut *DFU.as_mut_ptr() }.reboot_into(),
                Some("reboot") => cortex_m::peripheral::SCB::sys_reset(),
                Some("telemetry") => console!(serial, "{}\n", telemetry.get_data()),
                _ => return false,
            }
            true
        });
    }
}

#[panic_handler]
unsafe fn panic(panic_info: &PanicInfo) -> ! {
    log_panic(format_args!("{}", panic_info), panic_logger!());
    (&mut *DFU.as_mut_ptr()).reboot_into();
    loop {}
}

#[exception]
unsafe fn HardFault(exception_frame: &ExceptionFrame) -> ! {
    log_panic(format_args!("{:?}", exception_frame), panic_logger!());
    (&mut *DFU.as_mut_ptr()).reboot_into();
    loop {}
}

#[exception]
unsafe fn DefaultHandler(irqn: i16) {
    log_panic(format_args!("{}", irqn), panic_logger!());
    (&mut *DFU.as_mut_ptr()).reboot_into();
}
