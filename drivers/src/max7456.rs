use alloc::boxed::Box;
use core::future::Future;

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use hal::dma::{BufferDescriptor, TransferOption, DMA};
use max7456::{lines_writer::LinesWriter, registers::Standard, MAX7456};
use pro_flight::{
    components::{ascii_hud::AsciiHud, flight_data::FlightDataReader},
    config,
    datastructures::Ratio,
    sys::time::TickTimer,
};

pub fn init<E, PE, SPI, CS>(spi: SPI, cs: CS) -> Result<MAX7456<SPI, CS>, E>
where
    SPI: Write<u8, Error = E> + Transfer<u8, Error = E>,
    CS: OutputPin<Error = PE>,
{
    let config = &config::get().osd;
    let mut max7456 = MAX7456::new(spi, cs);
    let mut delay = TickTimer::default();
    max7456.reset(&mut delay)?;
    let standard = match config.standard {
        config::Standard::PAL => Standard::PAL,
        config::Standard::NTSC => Standard::NTSC,
    };
    max7456.set_standard(standard)?;
    if config.offset.horizental != 0 {
        max7456.set_horizental_offset(config.offset.horizental)?;
    }
    if config.offset.vertical != 0 {
        max7456.set_vertical_offset(config.offset.vertical)?;
    }
    max7456.enable_display(true)?;
    Ok(max7456)
}

pub struct DmaMAX7456<'a, CS, TX> {
    cs: CS,
    tx: TX,
    reader: FlightDataReader<'a>,
    bd: Box<BufferDescriptor<u8, 800>>,
}

pub trait IntoDMA<'a, O, CS, TXF, TX>
where
    TXF: Future<Output = O>,
    TX: DMA<Future = TXF>,
{
    type Error;
    fn into_dma(
        self,
        tx: TX,
        reader: FlightDataReader<'a>,
    ) -> Result<DmaMAX7456<'a, CS, TX>, Self::Error>;
}

impl<'a, E, PE, SPI, CS, O, TXF, TX> IntoDMA<'a, O, CS, TXF, TX> for MAX7456<SPI, CS>
where
    SPI: Write<u8, Error = E> + Transfer<u8, Error = E>,
    CS: OutputPin<Error = PE> + Send + 'static,
    TXF: Future<Output = O>,
    TX: DMA<Future = TXF>,
{
    type Error = E;
    fn into_dma(self, tx: TX, reader: FlightDataReader<'a>) -> Result<DmaMAX7456<'a, CS, TX>, E> {
        let mut bd = Box::new(BufferDescriptor::<u8, 800>::default());
        let (_, cs) = self.free();
        let mut cs_ = unsafe { core::ptr::read(&cs as *const _ as *const CS) };
        bd.set_transfer_done(move |_bytes| {
            cs_.set_high().ok();
        });
        Ok(DmaMAX7456 { cs, tx, reader, bd })
    }
}

impl<'a, E, O, CS, TXF, TX> DmaMAX7456<'a, CS, TX>
where
    CS: OutputPin<Error = E> + Send + 'static,
    TXF: Future<Output = O>,
    TX: DMA<Future = TXF>,
{
    pub async fn run(mut self) {
        let mut hud = AsciiHud::<29, 15>::new(self.reader, Ratio(12, 18).into());
        loop {
            let buffer = self.bd.try_get_buffer().unwrap();
            let screen = hud.draw();
            let mut writer = LinesWriter::new(screen, Default::default());
            let size = writer.write(buffer).0.len();
            self.cs.set_low().ok();
            self.tx.tx(&self.bd, TransferOption::sized(size)).await;
        }
    }
}
