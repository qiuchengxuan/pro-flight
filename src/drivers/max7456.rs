use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::{Transfer, Write};
use max7456::not_null_writer::NotNullWriter;
use max7456::registers::{Standard, SyncMode};
use max7456::MAX7456;

type DmaConsumer = fn(&[u8]);

pub fn init<BUS, E>(bus: BUS, delay: &mut dyn DelayMs<u8>) -> Result<(), E>
where
    BUS: Write<u8, Error = E> + Transfer<u8, Error = E>,
{
    let mut max7456 = MAX7456::new(bus);
    max7456.reset(delay)?;
    max7456.set_standard(Standard::PAL)?;
    max7456.set_sync_mode(SyncMode::Internal)?;
    max7456.set_horizental_offset(8)?;
    max7456.enable_display(true)?;
    Ok(())
}

pub fn process_screen<T: AsRef<[u8]>>(screen: &[T], dma_consumer: DmaConsumer) {
    static mut S_DMA_BUFFER: [u8; 1000] = [0u8; 1000];
    let mut dma_buffer = unsafe { S_DMA_BUFFER };
    let mut writer = NotNullWriter::new(screen, Default::default());
    let display = writer.write(&mut dma_buffer);
    dma_consumer(&display.0);
}
