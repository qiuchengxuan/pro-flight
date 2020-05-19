use max7456::registers::{Standard, SyncMode};
use max7456::{MAX7456, SPI_MODE};
use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Error, Pins, Spi};
use stm32f4xx_hal::{prelude::*, stm32};

pub fn init<'a, PINS: Pins<stm32::SPI3>>(
    spi3: stm32::SPI3,
    pins: PINS,
    clocks: Clocks,
    delay: &mut Delay,
) -> Result<MAX7456<Spi<stm32::SPI3, PINS>>, Error> {
    let freq: stm32f4xx_hal::time::Hertz = 10.mhz().into();
    let spi3 = Spi::spi3(spi3, pins, SPI_MODE, freq, clocks);
    let mut max7456 = MAX7456::new(spi3);
    max7456.clear_display(delay)?;
    max7456.set_standard(Standard::PAL)?;
    max7456.set_sync_mode(SyncMode::Internal)?;
    max7456.enable_display(true)?;
    Ok(max7456)
}
