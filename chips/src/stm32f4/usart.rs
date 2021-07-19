use hal::dma::Peripheral;

use stm32f4xx_hal::{
    dma::traits::PeriAddress,
    serial::{Pins, Serial},
    stm32,
};

pub struct UsartPeripheral {
    index: usize,
    address: usize,
}

impl core::fmt::Display for UsartPeripheral {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "USART{}", self.index)
    }
}

impl Peripheral for UsartPeripheral {
    fn enable_dma(&mut self) {}

    fn address(&mut self) -> usize {
        self.address
    }

    fn word_size(&self) -> usize {
        core::mem::size_of::<u8>()
    }
}

pub trait IntoDMA {
    fn into_dma(self) -> UsartPeripheral;
}

macro_rules! dma_usart {
    ($index:ident, $type:ty) => {
        impl<PINS: Pins<$type>> IntoDMA for Serial<$type, PINS> {
            fn into_dma(self) -> UsartPeripheral {
                let (tx, _) = self.split();
                UsartPeripheral{$index, tx.address() as usize}
            }
        }
    };
}

dma_usart!(1, stm32::USART1);
dma_usart!(6, stm32::USART6);
