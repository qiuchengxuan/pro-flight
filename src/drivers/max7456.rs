use max7456::not_null_writer::NotNullWriter;
use max7456::registers::{Standard, SyncMode};
use max7456::MAX7456;

pub struct Max7456Dma(MAX7456);

pub fn init<BUS, E>(max7456: &mut MAX7456<BUS>, delay: &mut dyn DelayMs<u8>) -> Result<(), E>
where
    BUS: Write<u8, Error = E> + Transfer<u8, Error = E>,
{
    max7456.reset(delay)?;
    max7456.set_standard(Standard::PAL)?;
    max7456.set_sync_mode(SyncMode::Internal)?;
    max7456.set_horizental_offset(8)?;
    max7456.enable_display(true)
}

type DmaConsumer = fn(&[u8]);

pub fn dma_draw(&mut self) {
    // ascii-hud will generator about 120 chars, for each char
    // max7456 will generate 4 byte to write, so at lease 480 bytes
    // memory space is required
    static mut S_DMA_BUFFER: [u8; 800] = [0u8; 800];
    let mut dma_buffer = unsafe { S_DMA_BUFFER };
    self.hud.draw(&mut self.screen);
    let mut writer = NotNullWriter::new(&self.screen, Default::default());
    let display = writer.write(&mut dma_buffer);
    (self.dma_consumer)(&display.0);
}
