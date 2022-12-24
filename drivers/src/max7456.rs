use alloc::boxed::Box;
use core::{future::Future, mem, ptr};

use embedded_hal::{
    blocking::spi::{Transfer, Write},
    digital::v2::OutputPin,
};
use fugit::NanosDurationU32 as Duration;
use hal::dma::{BufferDescriptor, Error, TransferOption, TransferResult, DMA};
use max7456::{
    character_memory::{build_store_char_operation, CHAR_DATA_SIZE, STORE_CHAR_BUFFER_SIZE},
    lines_writer::LinesWriter,
    registers::{Registers, Standard, VideoMode0},
    MAX7456,
};
use peripheral_register::Register;
use pro_flight::{
    config, io,
    osd::ascii::OSD,
    protocol::xmodem::XMODEM,
    sync::event::{Event, Subscriber},
    sys::time::TickTimer,
    types::Ratio,
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

pub struct DmaMAX7456<CS, TX> {
    cs: CS,
    event: Event,
    tx: TX,
    video_mode_0: Register<u8, VideoMode0>,
    bd: Box<BufferDescriptor<u8, 800>>,
}

pub trait IntoDMA<O, CS, TXF, TX>
where
    TXF: Future<Output = O>,
    TX: DMA<Future = TXF>,
{
    type Error;
    fn into_dma(self, event: Event, tx: TX) -> Result<DmaMAX7456<CS, TX>, Self::Error>;
}

impl<E, PE, SPI, CS, O, TXF, TX> IntoDMA<O, CS, TXF, TX> for MAX7456<SPI, CS>
where
    SPI: Write<u8, Error = E> + Transfer<u8, Error = E>,
    CS: OutputPin<Error = PE> + Send + 'static,
    TXF: Future<Output = O>,
    TX: DMA<Future = TXF>,
{
    type Error = E;

    fn into_dma(mut self, event: Event, tx: TX) -> Result<DmaMAX7456<CS, TX>, E> {
        let video_mode_0 = self.load(Registers::VideoMode0)?;
        let (_, cs) = self.free();
        let mut cs_ = unsafe { ptr::read(ptr::addr_of!(cs)) };
        let callback = Box::leak(Box::new(move |_: TransferResult<u8>| {
            cs_.set_high().ok();
        }));
        let bd = Box::new(BufferDescriptor::<u8, 800>::with_callback(callback));
        Ok(DmaMAX7456 { cs, event, tx, video_mode_0, bd })
    }
}

impl<E, O, CS, TXF, TX> DmaMAX7456<CS, TX>
where
    CS: OutputPin<Error = E> + Send + 'static,
    TXF: Future<Output = O>,
    TX: DMA<Future = TXF>,
{
    async fn enable_display(&mut self, enable: bool) {
        let mut video_mode_0 = self.video_mode_0;
        video_mode_0.set(VideoMode0::EnableDisplay, enable as u8);
        let mut buffer = self.bd.as_mut().cpu_try_take().unwrap();
        buffer[0] = Registers::VideoMode0 as u8;
        buffer[1] = video_mode_0.value;
        mem::drop(buffer);
        self.cs.set_low().ok();
        self.tx.tx(&self.bd, TransferOption::default().size(2)).unwrap().await;
    }

    async fn upload_char(&mut self, bytes: &[u8], index: u8) {
        let mut buffer = self.bd.as_mut().cpu_try_take().unwrap();
        let mut char_data = [0u8; CHAR_DATA_SIZE];
        char_data.copy_from_slice(bytes);
        build_store_char_operation(&char_data, index, buffer.as_mut());
        mem::drop(buffer);
        let timer = TickTimer::after(Duration::millis(13));
        self.cs.set_low().ok();
        self.tx.tx(&self.bd, TransferOption::default().size(STORE_CHAR_BUFFER_SIZE)).unwrap().await;
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

impl<E, O, CS, TXF, TX> DmaMAX7456<CS, TX>
where
    CS: OutputPin<Error = E> + Send + 'static,
    TXF: Future<Output = O>,
    TX: DMA<Future = TXF>,
{
    pub async fn run(mut self) {
        let mut frame_buf = [[0u8; 29]; 15];
        let osd = OSD::new(Ratio(12, 18).into());
        loop {
            if self.event.wait() {
                self.upload_font().await;
                self.event.clear();
            }
            let size = match self.bd.cpu_try_take() {
                Ok(mut buffer) => {
                    let frame = osd.draw(&mut frame_buf);
                    let mut writer = LinesWriter::new(frame, Default::default());
                    writer.write(buffer.as_mut()).0.len()
                }
                _ => 0,
            };
            if size == 0 {
                continue;
            }
            self.cs.set_low().ok();
            match self.tx.tx(&self.bd, TransferOption::default().size(size)) {
                Ok(future) => future.await,
                Err(Error::Busy) => {
                    TickTimer::after(Duration::millis(1)).await;
                    continue;
                }
                Err(e) => panic!("DMA error: {:?}", e),
            };
            self.cs.set_high().ok();
        }
    }
}
