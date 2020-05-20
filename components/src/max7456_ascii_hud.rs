use ascii_osd_hud::hud::HUD;
use ascii_osd_hud::symbol::{Symbol, SymbolTable};
use ascii_osd_hud::telemetry::{Telemetry, TelemetrySource};
use ascii_osd_hud::AspectRatio;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::blocking::spi::{Transfer, Write};
use max7456::not_null_writer::{revert, NotNullWriter};
use max7456::registers::{Standard, SyncMode};
use max7456::MAX7456;

// ascii-hud will generator about 120 chars, for each char
// max7456 will generate 4 byte to write, so at lease 480 bytes
// memory space is required
static mut DMA_BUFFER: [u8; 1000] = [0u8; 1000];

type DmaConsumer = fn(&[u8]);

pub struct Max7456AsciiHud<'a, BUS> {
    hud: HUD<'a>,
    max7456: MAX7456<BUS>,
    dma_consumer: DmaConsumer,
}

pub struct StubTelemetrySource {}

impl TelemetrySource for StubTelemetrySource {
    fn get_telemetry(&self) -> Telemetry {
        Default::default()
    }
}

impl<'a, E, BUS> Max7456AsciiHud<'a, BUS>
where
    BUS: Write<u8, Error = E> + Transfer<u8, Error = E>,
{
    pub fn new(
        telemetry: &'a dyn TelemetrySource,
        max7456: MAX7456<BUS>,
        dma_consumer: DmaConsumer,
    ) -> Self {
        let symbol_table: SymbolTable = enum_map! {
            Symbol::Antenna => 1,
            Symbol::Battery => 144,
            Symbol::Degree => 168,
            Symbol::CrossHair => 126,
            Symbol::VeclocityVector => 132,
            Symbol::Alpha => 154,
            Symbol::Square => 191,
            Symbol::LineTop => 128, // ▔
            Symbol::LineUpper1 => 129, // ⎺
            Symbol::LineUpper2 => 130, // ⎻
            Symbol::LineCenter => 131, // ⎯ or ASCII dash
            Symbol::LineLower1 => 132, // ⎼
            Symbol::LineLower2 => 133, // ⎽
            Symbol::LineBottom => 134, // ▁ or ASCII underscore
            Symbol::BoxDrawningLightUp => 124, // ╵ or ASCII |
            Symbol::ZeroWithTraillingDot => 192,
            Symbol::SmallBlackSquare => 46, // ▪
            Symbol::VerticalLine => 124, // ⎪
        };
        let hud = HUD::new(telemetry, &symbol_table, 150, AspectRatio::Standard);
        Self {
            hud,
            max7456,
            dma_consumer,
        }
    }

    pub fn init(&mut self, delay: &mut dyn DelayUs<u8>) -> Result<(), E> {
        self.max7456.wait_clear_display(delay)?;
        self.max7456.set_standard(Standard::PAL)?;
        self.max7456.set_sync_mode(SyncMode::Internal)?;
        self.max7456.set_horizental_offset(8)?;
        self.max7456.enable_display(true)
    }

    pub fn start_draw(&mut self) {
        let offset = revert(unsafe { &mut DMA_BUFFER }).0.len();
        let mut screen = [[0u8; 29]; 16];
        self.hud.draw(&mut screen);
        let mut writer = NotNullWriter::new(&screen, Default::default());
        let operations = writer.write(unsafe { &mut DMA_BUFFER[offset..] });
        let consumer = self.dma_consumer;
        consumer(operations.0);
    }
}
