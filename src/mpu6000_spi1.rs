use core::convert::Infallible;
use mpu6000::{SpiBus, SpiError, MPU6000, SPI_MODE};
use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::gpio::gpioa::PA4;
use stm32f4xx_hal::gpio::{Output, PushPull};
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Error, Pins, Spi};
use stm32f4xx_hal::{prelude::*, stm32};

pub fn init<'a, PINS: Pins<stm32::SPI1>>(
    spi1: stm32::SPI1,
    pins: PINS,
    cs: PA4<Output<PushPull>>,
    clocks: Clocks,
    delay: &mut Delay,
) -> Result<bool, SpiError<Error, Error, Infallible>> {
    let freq: stm32f4xx_hal::time::Hertz = 1.mhz().into();
    let spi1 = Spi::spi1(spi1, pins, SPI_MODE, freq, clocks);
    let mut bus = SpiBus::new(spi1, cs);
    let mut mpu6000 = MPU6000::new(&mut bus);
    mpu6000.reset(delay)?;
    if mpu6000.verify()? {
        return Ok(true);
    }
    Ok(false)
}
