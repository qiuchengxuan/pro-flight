use embedded_hal::watchdog::Watchdog;
use stm32f4xx_hal::time::MilliSeconds;
use stm32f4xx_hal::watchdog::IndependentWatchdog;
use stm32f4xx_hal::{prelude::*, stm32};

pub fn init(watchdog: stm32::IWDG) -> impl Watchdog {
    let mut watchdog = IndependentWatchdog::new(watchdog);
    watchdog.start(MilliSeconds(1000));
    watchdog
}
