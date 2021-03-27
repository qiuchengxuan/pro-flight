use alloc::boxed::Box;
use core::future::Future;

use crc::Hasher32;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use hal::dma::{BufferDescriptor, TransferOption, DMA};
use max7456::character_memory::{CharData, CHAR_DATA_SIZE};
use max7456::lines_writer::LinesWriter;
use max7456::registers::Standard;
use max7456::MAX7456;
use pro_flight::components::{ascii_hud::AsciiHud, flight_data::FlightDataReader};
use pro_flight::config;
use pro_flight::datastructures::Ratio;
use pro_flight::sys::timer::SysTimer;

pub trait HashFont {
    type Error;
    fn hash_font_crc32<CRC: Hasher32>(&mut self, crc: &mut CRC) -> Result<u32, Self::Error>;
}

impl<E, PE, BUS, CS> HashFont for MAX7456<BUS, CS>
where
    BUS: Write<u8, Error = E> + Transfer<u8, Error = E>,
    CS: OutputPin<Error = PE>,
{
    type Error = E;
    fn hash_font_crc32<CRC: Hasher32>(&mut self, crc: &mut CRC) -> Result<u32, E> {
        self.enable_display(false)?;
        crc.reset();
        let mut char_data: CharData = [0u8; CHAR_DATA_SIZE];
        for i in 0..256 {
            self.load_char(i as u8, &mut char_data)?;
            crc.write(&char_data);
        }
        self.enable_display(true)?;
        Ok(crc.sum32())
    }
}

pub fn init<E, PE, BUS, CS>(bus: BUS, cs: CS) -> Result<MAX7456<BUS, CS>, E>
where
    BUS: Write<u8, Error = E> + Transfer<u8, Error = E>,
    CS: OutputPin<Error = PE>,
{
    let config = &config::get().osd;
    let mut max7456 = MAX7456::new(bus, cs);
    let mut delay = SysTimer::new();
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

pub struct DmaMAX7456<'a, CS> {
    cs: CS,
    reader: FlightDataReader<'a>,
    bd: Box<BufferDescriptor<u8, 800>>,
}

impl<'a, E, CS: OutputPin<Error = E> + Send + 'static> DmaMAX7456<'a, CS> {
    pub fn new(cs: CS, reader: FlightDataReader<'a>) -> Self {
        let mut bd = Box::new(BufferDescriptor::<u8, 800>::default());
        let mut cs_ = unsafe { core::ptr::read(&cs as *const _ as *const CS) };
        bd.set_transfer_done(move |_bytes| {
            cs_.set_high().ok();
        });
        Self { cs, reader, bd }
    }
}

impl<'a, E, CS: OutputPin<Error = E> + Send + Unpin + 'static> DmaMAX7456<'a, CS> {
    pub async fn run<O, RXF, RX, TXF, TX>(mut self, _rx: RX, tx: TX)
    where
        RXF: Future<Output = O>,
        RX: DMA<Future = RXF>,
        TXF: Future<Output = O>,
        TX: DMA<Future = TXF>,
    {
        let mut hud = AsciiHud::<29, 15>::new(self.reader, Ratio(12, 18).into());
        loop {
            let buffer = self.bd.try_get_buffer().unwrap();
            let screen = hud.draw();
            let mut writer = LinesWriter::new(screen, Default::default());
            let size = writer.write(buffer).0.len();
            self.cs.set_low().ok();
            tx.tx(&self.bd, TransferOption::sized(size)).await;
        }
    }
}
