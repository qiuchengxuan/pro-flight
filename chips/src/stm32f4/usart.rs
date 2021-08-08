use hal::dma::Peripheral;

use embedded_hal::serial;
use stm32f4xx_hal::{
    dma::traits::PeriAddress,
    serial::{Error, Pins, Rx, Serial},
    stm32,
};

pub struct UsartPeripheral<RX> {
    name: &'static str,
    rx: RX,
    address: usize,
}

impl<RX> core::fmt::Display for UsartPeripheral<RX> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.name.split("::").last().unwrap())
    }
}

impl<RX> serial::Read<u8> for UsartPeripheral<RX>
where
    RX: serial::Read<u8, Error = Error>,
{
    type Error = hal::serial::Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.rx.read().map_err(|e| {
            e.map(|e| match e {
                Error::Framing => hal::serial::Error::Framing,
                Error::Noise => hal::serial::Error::Noise,
                Error::Overrun => hal::serial::Error::Overrun,
                Error::Parity => hal::serial::Error::Parity,
                _ => hal::serial::Error::Unknown,
            })
        })
    }
}

impl<RX> Peripheral for UsartPeripheral<RX> {
    fn enable_dma(&mut self) {}

    fn address(&mut self) -> usize {
        self.address
    }

    fn word_size(&self) -> usize {
        core::mem::size_of::<u8>()
    }
}

pub trait IntoDMA<RX: serial::Read<u8, Error = Error>> {
    fn into_dma(self) -> UsartPeripheral<RX>;
}

macro_rules! dma_usart {
    ($index:expr, $type:ty) => {
        impl<PINS: Pins<$type>> IntoDMA<Rx<$type>> for Serial<$type, PINS> {
            fn into_dma(self) -> UsartPeripheral<Rx<$type>> {
                let (tx, rx) = self.split();
                let name = core::any::type_name::<$type>();
                UsartPeripheral { name, rx, address: tx.address() as usize }
            }
        }
    };
}

dma_usart!(1, stm32::USART1);
dma_usart!(6, stm32::USART6);
