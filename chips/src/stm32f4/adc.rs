use hal::dma::Peripheral;

use stm32f4xx_hal::{adc::Adc, stm32};

pub struct AdcPeripheral(usize);

impl Peripheral for AdcPeripheral {
    fn enable_dma(&mut self) {}

    fn address(&mut self) -> usize {
        self.0
    }

    fn word_size(&self) -> usize {
        core::mem::size_of::<u16>()
    }
}

pub trait IntoDMA {
    fn into_dma(self) -> AdcPeripheral;
}

impl IntoDMA for Adc<stm32::ADC2> {
    fn into_dma(mut self) -> AdcPeripheral {
        AdcPeripheral(self.data_register_address() as usize)
    }
}
