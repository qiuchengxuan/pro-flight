use alloc::boxed::Box;
use core::future::Future;

pub use bmp280::DEFAULT_SPI_MODE as SPI_MODE;
use bmp280::{
    bus::{Bus, SpiBus},
    measurement::{Calibration, RawPressure, RawTemperature},
    registers::{PressureOversampling, Register, StandbyTime, TemperatureOversampling},
    Mode, BMP280,
};
use embedded_hal::{
    blocking::{
        delay::DelayUs,
        spi::{Transfer, Write},
    },
    digital::v2::OutputPin,
};
use fugit::NanosDurationU32 as Duration;
use hal::dma::{BufferDescriptor, Error, TransferOption, DMA};
use pro_flight::{sys::time::TickTimer, types::measurement::Pressure};

pub const SAMPLE_RATE: usize = 16;

pub struct Compensator(pub Calibration);

impl Compensator {
    pub fn convert(&self, bytes: &[u8]) -> Pressure {
        let raw_pressure = RawPressure::from_bytes(&bytes[2..]);
        let t_fine = RawTemperature::from_bytes(&bytes[5..]).t_fine(&self.0);
        Pressure(raw_pressure.compensated(t_fine, &self.0))
    }
}

pub fn bmp280_spi<E, SPI, PE, CS, D>(spi: SPI, cs: CS, delay: D) -> BMP280<SpiBus<SPI, CS, D>>
where
    SPI: Transfer<u8, Error = E> + Write<u8, Error = E>,
    CS: OutputPin<Error = PE>,
    D: DelayUs<u8>,
{
    BMP280::new(SpiBus::new(spi, cs, delay))
}

pub trait BMP280Init<E> {
    fn init(&mut self) -> Result<(), E>;
}

impl<E, BUS: Bus<Error = E>> BMP280Init<E> for BMP280<BUS> {
    fn init(&mut self) -> Result<(), E> {
        let mut delay = TickTimer::default();
        self.reset(&mut delay)?;
        self.set_pressure_oversampling(PressureOversampling::StandardResolution)?;
        self.set_temperature_oversampling(TemperatureOversampling::UltraLowPower)?;
        self.set_mode(Mode::Normal)?;
        self.set_standby_time(StandbyTime::Hertz16)?;
        self.set_iir_filter(8)
    }
}

pub struct DmaBMP280<RX, TX, CS> {
    rx: RX,
    tx: TX,
    cs: CS,
    rx_bd: Box<BufferDescriptor<u8, 8>>,
    tx_bd: Box<BufferDescriptor<u8, 1>>,
    compensator: Compensator,
}

impl<E, O, RXF, TXF, RX, TX, CS> DmaBMP280<RX, TX, CS>
where
    RXF: Future<Output = O>,
    TXF: Future<Output = O>,
    RX: DMA<Future = RXF>,
    TX: DMA<Future = TXF>,
    CS: OutputPin<Error = E> + Send + 'static,
{
    pub fn new(rx: RX, tx: TX, cs: CS, compensator: Compensator) -> Self {
        let mut rx_bd = Box::new(BufferDescriptor::<u8, 8>::default());
        let address = rx_bd.cpu_try_take().unwrap().as_ptr();
        trace!("Init BMP280 DMA address at 0x{:x}", address as usize);
        let tx_bd = Box::new(BufferDescriptor::<u8, 1>::new([Register::PressureMsb as u8 | 0x80]));
        Self { rx, tx, cs, rx_bd, tx_bd, compensator }
    }

    pub async fn run(mut self, mut handler: impl FnMut(Pressure)) {
        loop {
            let future = match self.rx.rx(&mut self.rx_bd, Default::default()) {
                Ok(future) => future,
                Err(Error::Busy) => {
                    TickTimer::after(Duration::millis(1)).await;
                    continue;
                }
                Err(e) => panic!("DMA error: {:?}", e),
            };
            self.cs.set_low().ok();
            match self.tx.tx(&self.tx_bd, TransferOption::repeat().size(8)) {
                Ok(_) => (),
                Err(Error::Busy) => {
                    TickTimer::after(Duration::millis(1)).await;
                    self.rx.stop();
                    continue;
                }
                Err(e) => panic!("DMA error: {:?}", e),
            }
            future.await;
            self.cs.set_high().ok();
            if let Some(buffer) = self.rx_bd.cpu_try_take().ok() {
                handler(self.compensator.convert(&buffer));
            }
            TickTimer::after(Duration::micros(62_500)).await;
        }
    }
}
