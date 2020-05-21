use core::convert::Infallible;

use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::gpio::gpioa::{PA4, PA6, PA7, PA8};
use stm32f4xx_hal::gpio::{Alternate, AF5};
use stm32f4xx_hal::gpio::{Output, PushPull};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Error, Pins, Spi};
use stm32f4xx_hal::timer::{Event, Timer};
use stm32f4xx_hal::{prelude::*, stm32};

use mpu6000::registers::Register;
use mpu6000::{FifoEnable, SpiBus, SpiError, MPU6000, SPI_MODE};

type Spi1Pins = (
    PA6<Alternate<AF5>>,
    PA7<Alternate<AF5>>,
    PA8<Alternate<AF5>>,
);

static G_FIFO_READ: [u8; 1] = [Register::FifoReadWrite as u8; 1];
static mut G_TIM6: Option<Timer<stm32::TIM6>> = None;
static mut G_MPU6000: Option<MPU6000<Spi<stm32::SPI3, Spi1Pins>>> = None;

#[interrupt]
fn TIM6_DAC() {
    cortex_m::interrupt::free(|_cs| unsafe {
        if let Some(ref mut tim) = G_TIM6 {
            tim.clear_interrupt(Event::TimeOut);
        };
    });
}

fn dma2_spi1_stream2_receive(buffer: &mut [u8]) {}

pub fn init<'a, PINS: Pins<stm32::SPI1>>(
    spi1: stm32::SPI1,
    tim6: stm32::TIM6,
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
    if !mpu6000.verify()? {
        return Ok(false);
    }

    mpu6000.enable_fifo(FifoEnable {
        temperature: true,
        x_g_force: true,
        y_g_force: true,
        z_g_force: true,
        acceleration: true,
        ..Default::default()
    })?;
    mpu6000.enable_fifo_buffer()?;
    Ok(true)
}
