use alloc::boxed::Box;

use bmp280::bus::{Bus, SpiBus};
use bmp280::measurement::{Calibration, RawPressure, RawTemperature};
use bmp280::registers::{PressureOversampling, Register, StandbyTime, TemperatureOversampling};
pub use bmp280::DEFAULT_SPI_MODE as SPI_MODE;
use bmp280::{Mode, BMP280};
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use hal::dma::{BufferDescriptor, TransferOption, DMA};
use pro_flight::datastructures::measurement::Pressure;
use pro_flight::sys::timer::SysTimer;

pub const SAMPLE_RATE: usize = 10; // actually 16

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
        let mut delay = SysTimer::new();
        self.reset(&mut delay)?;
        self.set_pressure_oversampling(PressureOversampling::StandardResolution)?;
        self.set_temperature_oversampling(TemperatureOversampling::UltraLowPower)?;
        self.set_mode(Mode::Normal)?;
        self.set_standby_time(StandbyTime::Hertz16)?;
        self.set_iir_filter(8)
    }
}

pub struct DmaBMP280<CS> {
    rx_bd: Box<BufferDescriptor<u8, 8>>,
    tx_bd: Box<BufferDescriptor<u8, 1>>,
    cs: CS,
}

impl<E, CS: OutputPin<Error = E> + Send + Unpin + 'static> DmaBMP280<CS> {
    pub fn new<H>(cs: CS, compensator: Compensator, mut handler: H) -> Self
    where
        H: FnMut(Pressure) + Send + 'static,
    {
        let mut rx_bd = Box::new(BufferDescriptor::<u8, 8>::default());
        let address = rx_bd.try_get_buffer().unwrap().as_ptr();
        debug!("Init BMP280 DMA address at 0x{:x}", address as usize);
        let mut cs_ = unsafe { core::ptr::read(&cs as *const _ as *const CS) };
        rx_bd.set_transfer_done(move |bytes| {
            cs_.set_high().ok();
            handler(compensator.convert(&bytes[..]))
        });
        let tx_bd = Box::new(BufferDescriptor::<u8, 1>::new([Register::PressureMsb as u8 | 0x80]));
        Self { rx_bd, tx_bd, cs }
    }

    pub fn trigger<RXF, TXF, RX, TX>(&mut self, rx: &RX, tx: &TX)
    where
        RX: DMA<Future = RXF>,
        TX: DMA<Future = TXF>,
    {
        self.cs.set_low().ok();
        if rx.is_busy() || tx.is_busy() {
            return;
        }
        rx.rx(&self.rx_bd, Default::default());
        tx.tx(&self.tx_bd, TransferOption::repeat(8));
    }
}
