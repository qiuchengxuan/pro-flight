use core::convert::Infallible;
use core::mem::MaybeUninit;

use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::gpio::gpioa::{PA4, PA5, PA6, PA7};
use stm32f4xx_hal::gpio::gpioc::PC4;
use stm32f4xx_hal::gpio::ExtiPin;
use stm32f4xx_hal::gpio::{Alternate, AF5};
use stm32f4xx_hal::gpio::{Input, Output, PullUp, PushPull};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Error, Spi};
use stm32f4xx_hal::{prelude::*, stm32};

use rs_flight::components::mpu6000::init as mpu6000_init;
use rs_flight::datastructures::event::{event_nop_handler, EventHandler};
use rs_flight::hal::imu::AccelGyroHandler;
use rs_flight::hal::sensors;

use mpu6000::bus::{DelayNs, SpiBus};
use mpu6000::measurement::Measurement;
use mpu6000::registers::GyroSensitive;
use mpu6000::{self, MPU6000, SPI_MODE};

type Spi1Pins = (PA5<Alternate<AF5>>, PA6<Alternate<AF5>>, PA7<Alternate<AF5>>);

pub struct TickDelay(u32);

impl DelayNs<u8> for TickDelay {
    fn delay_ns(&mut self, ns: u8) {
        cortex_m::asm::delay(ns as u32 * (self.0 / 1000_000) / 1000 + 1)
    }
}

impl DelayNs<u16> for TickDelay {
    fn delay_ns(&mut self, ns: u16) {
        cortex_m::asm::delay(ns as u32 * (self.0 / 1000_000) / 1000 + 1)
    }
}

type SpiError = mpu6000::bus::SpiError<Error, Error, Infallible>;
type Spi1Bus<'a> = SpiBus<'a, Spi<stm32::SPI1, Spi1Pins>, PA4<Output<PushPull>>, TickDelay>;
static mut G_MPU6000: MaybeUninit<MPU6000<Spi1Bus>> = MaybeUninit::uninit();
static mut G_CS: MaybeUninit<PA4<Output<PushPull>>> = MaybeUninit::uninit();
static mut G_INT: MaybeUninit<PC4<Input<PullUp>>> = MaybeUninit::uninit();
static mut G_CALIBRATION: MaybeUninit<Measurement<GyroSensitive>> = MaybeUninit::uninit();

static mut G_ACCEL_GYRO_HANDLER: AccelGyroHandler = event_nop_handler;
static mut G_TEMPERATURE_HANDLER: EventHandler<sensors::Temperature<i16>> = event_nop_handler;

#[interrupt]
unsafe fn EXTI4() {
    { &mut *G_INT.as_mut_ptr() }.clear_interrupt_pending_bit();
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::EXTI4);
    let mpu6000 = &mut *G_MPU6000.as_mut_ptr();

    let accel = mpu6000.get_acceleration().ok().unwrap();
    let mut gyro = mpu6000.get_gyro().ok().unwrap();
    gyro.calibrated(&*G_CALIBRATION.as_ptr());
    G_ACCEL_GYRO_HANDLER((accel.into(), gyro.into()));
}

pub fn init(
    spi1: stm32::SPI1,
    pins: Spi1Pins,
    cs: PA4<Output<PushPull>>,
    int: PC4<Input<PullUp>>,
    clocks: Clocks,
    event_handlers: (AccelGyroHandler, EventHandler<sensors::Temperature<i16>>),
    delay: &mut Delay,
) -> Result<(), SpiError> {
    let freq: stm32f4xx_hal::time::Hertz = 1.mhz().into();
    let spi1 = Spi::spi1(spi1, pins, SPI_MODE, freq, clocks);
    unsafe { G_CS = MaybeUninit::new(cs) };
    let bus = SpiBus::new(spi1, unsafe { &mut *G_CS.as_mut_ptr() }, TickDelay(clocks.sysclk().0));
    let mut mpu6000 = MPU6000::new(bus);
    if !mpu6000_init(&mut mpu6000, delay)? {
        return Ok(());
    }

    let freq: stm32f4xx_hal::time::Hertz = 20.mhz().into();
    let br = match clocks.pclk2().0 / freq.0 {
        0 => unreachable!(),
        1..=2 => 0b000,
        3..=5 => 0b001,
        6..=11 => 0b010,
        12..=23 => 0b011,
        24..=47 => 0b100,
        48..=95 => 0b101,
        96..=191 => 0b110,
        _ => 0b011,
    };
    let spi1 = unsafe { &(*stm32::SPI1::ptr()) };
    spi1.cr1.modify(|_, w| w.br().bits(br));

    let mut calibration = mpu6000.get_gyro().ok().unwrap();
    for _ in 0..200 {
        delay.delay_ms(1u8);
        let gyro = mpu6000.get_gyro().ok().unwrap();
        calibration = Measurement::average(&calibration, &gyro);
    }

    let (accel_gyro_handler, temperature_handler) = event_handlers;
    unsafe {
        G_INT = MaybeUninit::new(int);
        G_ACCEL_GYRO_HANDLER = accel_gyro_handler;
        G_TEMPERATURE_HANDLER = temperature_handler;
        G_CALIBRATION = MaybeUninit::new(calibration);
    }

    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::EXTI4);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::EXTI4) }
    Ok(())
}
