use core::convert::Infallible;

use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::gpio::gpioa::{PA4, PA5, PA6, PA7};
use stm32f4xx_hal::gpio::{Alternate, AF5};
use stm32f4xx_hal::gpio::{Output, PushPull};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Error, Pins, Spi};
use stm32f4xx_hal::{prelude::*, stm32};

use rs_flight::datastructures::ring_buffer::RingBuffer;
use rs_flight::hal::sensors::{Acceleration, Gyro};

use mpu6000::registers::{AccelerometerSensitive, GyroSensitive, IntPinConfig};
use mpu6000::{self, SpiBus, MPU6000, SPI_MODE};

type Spi1Pins = (
    PA5<Alternate<AF5>>,
    PA6<Alternate<AF5>>,
    PA7<Alternate<AF5>>,
);

type SpiError = mpu6000::SpiError<Error, Error, Infallible>;
type Spi1Bus = SpiBus<Spi<stm32::SPI1, Spi1Pins>, PA4<Output<PushPull>>>;
static mut G_MPU6000: Option<MPU6000<Spi1Bus>> = None;

#[link_section = ".ram2bss"]
static mut G_GYRO_RING_BUFFER: Option<RingBuffer<Gyro<u32>>> = None;
#[link_section = ".ram2bss"]
static mut G_ACCELERATION_RING_BUFFER: Option<RingBuffer<Acceleration<u32>>> = None;
#[link_section = ".ram2bss"]
static mut G_TEMPERATURE_RING_BUFFER: Option<RingBuffer<u16>> = None;

#[interrupt]
fn EXTI4() {
    let exti = unsafe { &*(stm32::EXTI::ptr()) };
    cortex_m::interrupt::free(|_cs| exti.emr.modify(|_, w| w.mr4().set_bit()));
    static mut COUNTER: usize = 0;
    unsafe {
        if let Some(ref mut mpu6000) = G_MPU6000 {
            let gyro = mpu6000.get_gyro().ok().unwrap();
            if let Some(ref mut gyro_ring) = G_GYRO_RING_BUFFER {
                gyro_ring.push(Gyro(gyro));
            }
            let acceleration = mpu6000.get_acceleration().ok().unwrap();
            if let Some(ref mut acceleration_ring) = G_ACCELERATION_RING_BUFFER {
                acceleration_ring.push(Acceleration(acceleration));
            }
            // 10hz
            if COUNTER % 100 == 0 {
                if let Some(ref mut temperature_ring) = G_TEMPERATURE_RING_BUFFER {
                    temperature_ring.push(mpu6000.get_temperature().ok().unwrap());
                }
            }
            COUNTER += 0;
        }
    }
    exti.emr.modify(|_, w| w.mr4().clear_bit());
}

pub fn init<'a>(
    spi1: stm32::SPI1,
    pins: Spi1Pins,
    cs: PA4<Output<PushPull>>,
    clocks: Clocks,
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
    unsafe { G_MPU6000 = Some(mpu6000) };
    Ok(true)
}
