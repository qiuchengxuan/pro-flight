use core::cell::Cell;

use ascii_osd_hud::hud::HUD;
use ascii_osd_hud::symbol::{Symbol, SymbolTable};
use ascii_osd_hud::telemetry::{
    Attitude, SphericalCoordinate, Telemetry, TelemetrySource, Waypoint,
};
use ascii_osd_hud::AspectRatio;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::{Transfer, Write};
use max7456::not_null_writer::NotNullWriter;
use max7456::registers::{Standard, SyncMode};
use max7456::MAX7456;

type DmaConsumer = fn(&[u8]);

pub struct Max7456AsciiHud<'a> {
    hud: HUD<'a>,
    dma_consumer: DmaConsumer,
    screen: [[u8; 29]; 16],
}

pub struct StubTelemetrySource(pub Cell<u32>);

impl TelemetrySource for StubTelemetrySource {
    fn get_telemetry(&self) -> Telemetry {
        let value = self.0.get();
        self.0.set(value + 1);
        Telemetry {
            attitude: Attitude {
                roll: (value % 180 - 90) as i8,
                yaw: ((value / 25) % 360) as u16,
                ..Default::default()
            },
            waypoint: Waypoint {
                coordinate: SphericalCoordinate {
                    theta: (360 - (value / 25) % 360) as u16,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

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

impl<'a> Max7456AsciiHud<'a> {
    pub fn new(telemetry: &'a dyn TelemetrySource, dma_consumer: DmaConsumer) -> Self {
        let symbol_table: SymbolTable = enum_map! {
            Symbol::Antenna => 1,
            Symbol::Battery => 144,
            Symbol::Degree => 168,
            Symbol::CrossHair => 126,
            Symbol::VeclocityVector => 132,
            Symbol::Alpha => 154,
            Symbol::Square => 191,
            Symbol::LineTop => 128,
            Symbol::LineUpper1 => 129,
            Symbol::LineUpper2 => 131,
            Symbol::LineCenter => 132,
            Symbol::LineLower1 => 133,
            Symbol::LineLower2 => 134,
            Symbol::LineBottom => 136,
            Symbol::BoxDrawningLightUp => 124,
            Symbol::ZeroWithTraillingDot => 192,
            Symbol::LineLeft => 224,
            Symbol::LineLeft1 => 225,
            Symbol::LineVerticalCenter => 226,
            Symbol::LineRight => 227,
            Symbol::LineRight1 => 228,
        };
        let hud = HUD::new(telemetry, &symbol_table, 150, AspectRatio::Standard);
        Self {
            hud,
            dma_consumer,
            screen: [[0u8; 29]; 16],
        }
    }

    pub fn start_draw(&mut self) {
        // ascii-hud will generator about 120 chars, for each char
        // max7456 will generate 4 byte to write, so at lease 480 bytes
        // memory space is required
        static mut S_DMA_BUFFER: [u8; 500] = [0u8; 500];
        let mut dma_buffer = unsafe { S_DMA_BUFFER };
        self.hud.draw(&mut self.screen);
        let mut writer = NotNullWriter::new(&self.screen, Default::default());
        let display = writer.write(&mut dma_buffer);
        (self.dma_consumer)(&display.0);
    }
}
