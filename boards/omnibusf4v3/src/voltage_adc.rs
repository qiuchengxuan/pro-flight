use hal::dma::{BufferDescriptor, Peripheral, DMA};
use pro_flight::algorithm::lpf::LPF;
use pro_flight::datastructures::measurement::battery::Battery;
use stm32f4xx_hal::{
    adc::config::{AdcConfig, Continuous, Dma, SampleTime, Sequence},
    adc::Adc,
    gpio::{gpioc::PC2, Floating, Input},
    stm32,
};

pub struct VoltageADC(LPF<u16>);

impl Default for VoltageADC {
    fn default() -> Self {
        Self(LPF::<u16>::new(1.0, 5.0))
    }
}

const VOLTAGE_SCALE_X100: usize = 1100;
const SAMPLE_SIZE: usize = 16;
const VREF: usize = 3300;

impl VoltageADC {
    fn convert(&mut self, bytes: &[u16; SAMPLE_SIZE]) -> Battery {
        let sum: usize = bytes.iter().map(|&v| v as usize).sum();
        let value = self.0.filter((sum / SAMPLE_SIZE * VREF / 0xFFF) as u16) as usize;
        let milli_voltages = value * VOLTAGE_SCALE_X100 / 100;
        Battery(milli_voltages as u16)
    }
}

pub struct ADCWrapper(Adc<stm32::ADC2>);

impl Peripheral for ADCWrapper {
    fn enable_dma(&mut self) {}

    fn address(&mut self) -> usize {
        self.0.data_register_address() as usize
    }

    fn word_size(&self) -> usize {
        core::mem::size_of::<u16>()
    }
}

pub fn init<F, D, H>(adc2: stm32::ADC2, pc2: PC2<Input<Floating>>, mut dma: D, mut handler: H)
where
    D: DMA<Future = F>,
    H: FnMut(Battery) + 'static + Send,
{
    let config = AdcConfig::default().dma(Dma::Continuous).continuous(Continuous::Continuous);
    let mut adc = Adc::adc2(adc2, true, config);
    let vbat = pc2.into_analog();
    adc.configure_channel(&vbat, Sequence::One, SampleTime::Cycles_480);
    adc.start_conversion();

    let mut rx_bd = Box::new(BufferDescriptor::<u8, SAMPLE_SIZE>::default());
    let address = rx_bd.try_get_buffer().unwrap().as_ptr();
    info!("Init voltage ADC DMA address at {:x}", address as usize);
    let mut voltage_adc = VoltageADC::default();
    rx_bd.set_transfer_done(move |bytes| {
        let data = unsafe { &*(bytes.as_ptr() as *const _ as *const [u16; SAMPLE_SIZE]) };
        handler(voltage_adc.convert(data))
    });

    let mut adc = ADCWrapper(adc);
    dma.setup_peripheral(1, &mut adc);
    dma.rx(&rx_bd, true);
}
