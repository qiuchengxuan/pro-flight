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
) -> Result<bool, Error> {
    let freq: stm32f4xx_hal::time::Hertz = 10.mhz().into();
    let spi3 = Spi::spi3(spi3, pins, SPI_MODE, freq, clocks);
    let mut max7456 = MAX7456::new(spi3);
    let _ = delay.delay_ms(1000u16);
    max7456.reset(delay)?;
    max7456.set_standard(Standard::PAL)?;
    max7456.enable_display(true)?;
    max7456.set_sync_mode(SyncMode::Internal)?;
    max7456.display_line(8, 15, b"TEST", Default::default())?;
    Ok(true)
}
