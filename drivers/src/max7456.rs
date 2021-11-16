use alloc::boxed::Box;
use core::{future::Future, ptr, time::Duration};

use embedded_hal::{
    blocking::spi::{Transfer, Write},
    digital::v2::OutputPin,
};
use hal::dma::{BufferDescriptor, TransferOption, DMA};
use max7456::{
    character_memory::{build_store_char_operation, CHAR_DATA_SIZE, STORE_CHAR_BUFFER_SIZE},
    lines_writer::LinesWriter,
    registers::{Registers, Standard, VideoMode0},
    MAX7456,
};
use peripheral_register::Register;
use pro_flight::{
    components::{ascii_hud::AsciiHud, flight_data_hub::FlightDataReader},
    config,
    datastructures::Ratio,
    io,
    protocol::xmodem::XMODEM,
    sync::FlagReceiver,
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
    rx: FlagReceiver,
    tx: TX,
    reader: FlightDataReader<'a>,
    video_mode_0: Register<u8, VideoMode0>,
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
        rx: FlagReceiver,
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

    fn into_dma(
        mut self,
        rx: FlagReceiver,
        tx: TX,
        reader: FlightDataReader<'a>,
    ) -> Result<DmaMAX7456<'a, CS, TX>, E> {
        let video_mode_0 = self.load(Registers::VideoMode0)?;
        let mut bd = Box::new(BufferDescriptor::<u8, 800>::default());
        let (_, cs) = self.free();
        let mut cs_ = unsafe { ptr::read(ptr::addr_of!(cs)) };
        bd.set_callback(move |_bytes| {
            cs_.set_high().ok();
        });
        Ok(DmaMAX7456 { cs, rx, tx, reader, video_mode_0, bd })
    }
}

impl<'a, E, O, CS, TXF, TX> DmaMAX7456<'a, CS, TX>
where
    CS: OutputPin<Error = E> + Send + 'static,
    TXF: Future<Output = O>,
    TX: DMA<Future = TXF>,
{
    async fn enable_display(&mut self, enable: bool) {
        let mut video_mode_0 = self.video_mode_0;
        video_mode_0.set(VideoMode0::EnableDisplay, enable as u8);
        let buffer = self.bd.as_mut().try_get_buffer().unwrap();
        buffer[0] = Registers::VideoMode0 as u8;
        buffer[1] = video_mode_0.value;
        core::mem::drop(buffer);
        self.cs.set_low().ok();
        self.tx.tx(&self.bd, TransferOption::default().size(2)).await;
    }

    async fn upload_char(&mut self, bytes: &[u8], index: u8) {
        let buffer = self.bd.as_mut().try_get_buffer().unwrap();
        let mut char_data = [0u8; CHAR_DATA_SIZE];
        char_data.copy_from_slice(bytes);
        build_store_char_operation(&char_data, index, &mut buffer[..]);
        core::mem::drop(buffer);
        let timer = TickTimer::after(Duration::from_millis(13));
        self.cs.set_low().ok();
        self.tx.tx(&self.bd, TransferOption::default().size(STORE_CHAR_BUFFER_SIZE)).await;
        timer.await;
    }

    async fn upload_font(&mut self) {
        let mut stdin = io::stdin();
        stdin.lock();
        let mut stdout = io::stdout();
        let mut index = 0;
        self.enable_display(false).await;
        while let Some(bytes) = XMODEM::new(&mut stdin, &mut stdout).receive().await {
            self.upload_char(&bytes[..CHAR_DATA_SIZE], index).await;
            self.upload_char(&bytes[CHAR_DATA_SIZE..], index + 1).await;
            index += 2;
        }
        self.enable_display(true).await;
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
            if self.rx.get() {
                self.upload_font().await;
                self.rx.clear();
            }
            let buffer = self.bd.try_get_buffer().unwrap();
            let screen = hud.draw();
            let mut writer = LinesWriter::new(screen, Default::default());
            let size = writer.write(buffer).0.len();
            self.cs.set_low().ok();
            self.tx.tx(&self.bd, TransferOption::default().size(size)).await;
        }
    }
}
