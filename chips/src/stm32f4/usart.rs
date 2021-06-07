use hal::dma::Peripheral;

use stm32f4xx_hal::{
    dma::traits::PeriAddress,
    serial::{Pins, Serial},
    stm32,
};

pub struct UsartPeripheral(usize);

impl Peripheral for UsartPeripheral {
    fn enable_dma(&mut self) {}

    fn address(&mut self) -> usize {
        self.0
    }

    fn word_size(&self) -> usize {
        core::mem::size_of::<u8>()
    }
}

pub trait IntoDMA {
    fn into_dma(self) -> UsartPeripheral;
}

macro_rules! dma_usart {
    ($type:ty) => {
        impl<PINS: Pins<$type>> IntoDMA for Serial<$type, PINS> {
            fn into_dma(self) -> UsartPeripheral {
                let (tx, _) = self.split();
                UsartPeripheral(tx.address() as usize)
            }
        }
    };
}

dma_usart!(stm32::USART1);
dma_usart!(stm32::USART6);
