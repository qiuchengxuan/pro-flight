use alloc::boxed::Box;

use hal::dma::{BufferDescriptor, Peripheral, TransferOption, DMA};
use pro_flight::algorithm::lpf::LPF;
use pro_flight::datastructures::measurement::battery::Battery;
use stm32f4xx_hal::adc::config::{AdcConfig, Continuous, Dma, SampleTime, Sequence};

pub struct VoltageADC(LPF<u16>);

impl Default for VoltageADC {
    fn default() -> Self {
        Self(LPF::<u16>::new(1.0, 5.0))
    }
}

pub const SAMPLE_TIME: SampleTime = SampleTime::Cycles_480;
pub const SEQUENCE: Sequence = Sequence::One;

const VOLTAGE_SCALE_X100: usize = 1100;
const SAMPLE_SIZE: usize = 16;
const VREF: usize = 3300;

impl VoltageADC {
    fn convert(&mut self, data: &[u16]) -> Battery {
        let sum: usize = data.iter().map(|&v| v as usize).sum();
        let value = self.0.filter((sum / data.len() * VREF / 0xFFF) as u16) as usize;
        let milli_voltages = value * VOLTAGE_SCALE_X100 / 100;
        Battery(milli_voltages as u16)
    }
}

pub fn adc_config() -> AdcConfig {
    AdcConfig::default().dma(Dma::Continuous).continuous(Continuous::Continuous)
}

pub fn init<F, D, H>(mut adc: impl Peripheral, mut dma: D, mut handler: H)
where
    D: DMA<Future = F>,
    H: FnMut(Battery) + Send + 'static,
{
    let mut rx_bd = Box::new(BufferDescriptor::<u16, SAMPLE_SIZE>::default());
    let address = rx_bd.try_get_buffer().unwrap().as_ptr();
    trace!("Init voltage ADC DMA address at 0x{:x}", address as usize);
    let mut voltage_adc = VoltageADC::default();
    rx_bd.set_callback(move |result| handler(voltage_adc.convert(result.into())));

    dma.setup_peripheral(1, &mut adc);
    dma.setup_rx(Box::leak(rx_bd), TransferOption::circle());
}
