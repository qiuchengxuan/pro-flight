use core::time::Duration;

use embedded_hal::timer::CountDown;
use rs_flight::drivers::uart::probe;
use stm32f4xx_hal::gpio::gpioa;
use stm32f4xx_hal::gpio::{Floating, Input};
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::serial::config::Config;
use stm32f4xx_hal::serial::Serial;
use stm32f4xx_hal::{prelude::*, stm32};

type PA9 = gpioa::PA9<Input<Floating>>;
type PA10 = gpioa::PA10<Input<Floating>>;

pub fn init<C: CountDown<Time = Duration>>(
    usart1: stm32::USART1,
    pins: (PA9, PA10),
    baudrate: u32,
    clocks: Clocks,
    mut count_down: C,
) {
    debug!("Config USART1 to baudrate {}", baudrate);
    let (pa9, pa10) = pins;
    let pins = (pa9.into_alternate_af7(), pa10.into_alternate_af7());
    let config = Config::default().baudrate(baudrate.bps());
    let mut usart = Serial::usart1(usart1, pins, config, clocks).unwrap();
    if !probe(&mut usart, &mut count_down).ok().unwrap_or(false) {
        debug!("Received nothing on USART1");
        return;
    }
    debug!("Received something on USART1")
}
