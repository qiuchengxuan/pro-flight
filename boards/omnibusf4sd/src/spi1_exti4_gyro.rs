use core::convert::Infallible;
use core::mem::MaybeUninit;

use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::gpio::gpioa::{PA4, PA5, PA6, PA7};
use stm32f4xx_hal::gpio::{Alternate, AF5};
use stm32f4xx_hal::gpio::{Output, PushPull};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Error, Spi};
use stm32f4xx_hal::{prelude::*, stm32};

use rs_flight::datastructures::event::{event_nop_handler, EventHandler};
use rs_flight::hal::imu::AccelGyroHandler;
use rs_flight::hal::sensors::{Acceleration, Gyro, Temperature};

use mpu6000::registers::{AccelerometerSensitive, GyroSensitive, IntPinConfig};
use mpu6000::{self, SpiBus, MPU6000, SPI_MODE};

type Spi1Pins = (
    PA5<Alternate<AF5>>,
    PA6<Alternate<AF5>>,
    PA7<Alternate<AF5>>,
);

type SpiError = mpu6000::SpiError<Error, Error, Infallible>;
type Spi1Bus = SpiBus<Spi<stm32::SPI1, Spi1Pins>, PA4<Output<PushPull>>>;
static mut G_MPU6000: MaybeUninit<MPU6000<Spi1Bus>> = MaybeUninit::uninit();

static mut G_ACCEL_GYRO_HANDLER: AccelGyroHandler = event_nop_handler;
static mut G_TEMPERATURE_HANDLER: EventHandler<Temperature<u16>> = event_nop_handler;

#[interrupt]
fn EXTI4() {
    let exti = unsafe { &*(stm32::EXTI::ptr()) };
    cortex_m::interrupt::free(|_cs| exti.emr.modify(|_, w| w.mr4().set_bit()));
    static mut COUNTER: usize = 0;
    let mpu6000 = unsafe { &mut *G_MPU6000.as_mut_ptr() };

    static mut ACCELERATION_BUFFER: [Acceleration<u32>; 1] = [Acceleration((0, 0, 0)); 1];
    static mut GYRO_BUFFER: [Gyro<u32>; 1] = [Gyro((0, 0, 0)); 1];
    unsafe {
        ACCELERATION_BUFFER[0] = Acceleration(mpu6000.get_acceleration().ok().unwrap());
        GYRO_BUFFER[0] = Gyro(mpu6000.get_gyro().ok().unwrap());
        G_ACCEL_GYRO_HANDLER((&ACCELERATION_BUFFER, &GYRO_BUFFER));
    }

    unsafe {
        if COUNTER % 100 == 0 {
            let temperature = mpu6000.get_temperature().ok().unwrap();
            G_TEMPERATURE_HANDLER(Temperature(temperature));
        }
        COUNTER += 1;
    }
    exti.emr.modify(|_, w| w.mr4().clear_bit());
}

pub fn init(
    spi1: stm32::SPI1,
    pins: Spi1Pins,
    cs: PA4<Output<PushPull>>,
    clocks: Clocks,
    event_handlers: (AccelGyroHandler, EventHandler<Temperature<u16>>),
    delay: &mut Delay,
) -> Result<bool, SpiError> {
    let freq: stm32f4xx_hal::time::Hertz = 1.mhz().into();
    let spi1 = Spi::spi1(spi1, pins, SPI_MODE, freq, clocks);
    let bus = SpiBus::new(spi1, cs);
    let mut mpu6000 = MPU6000::new(bus);
    mpu6000.reset(delay)?;
    if !mpu6000.verify()? {
        return Ok(false);
    }
    mpu6000.set_sleep(false)?;
    mpu6000.set_accelerometer_sensitive(accelerometer_sensitive!(+/-4g, 8192/LSB))?;
    mpu6000.set_gyro_sensitive(gyro_sensitive!(+/-1000dps, 32.8LSB/dps))?;
    mpu6000.set_int_pin_config(IntPinConfig::IntReadClear, true)?;

    let freq: stm32f4xx_hal::time::Hertz = 20.mhz().into();
    let br = match clocks.pclk2().0 / freq.0 {
        0 => unreachable!(),
        1..=2 => 0b000,
        3..=5 => 0b001,
        6..=11 => 0b010,
        _ => 0b011,
    };
    let spi1 = unsafe { &(*stm32::SPI1::ptr()) };
    spi1.cr1.modify(|_, w| w.br().bits(br));
    unsafe { G_MPU6000 = MaybeUninit::new(mpu6000) };

    let (accel_gyro_handler, temperature_handler) = event_handlers;
    unsafe {
        G_ACCEL_GYRO_HANDLER = accel_gyro_handler;
        G_TEMPERATURE_HANDLER = temperature_handler;
    }

    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::EXTI4);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::EXTI4) }
    Ok(true)
}
