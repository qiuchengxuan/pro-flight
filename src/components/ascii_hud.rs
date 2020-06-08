use ascii_osd_hud::hud::HUD;
use ascii_osd_hud::symbol::{Symbol, SymbolTable};
use ascii_osd_hud::telemetry::TelemetrySource;
use ascii_osd_hud::AspectRatio;

pub type ScreenConsumer = fn(&[[u8; 29]; 16]);

pub struct AsciiHud<'a> {
    hud: HUD<'a>,
    screen: [[u8; 29]; 16],
}

impl<'a> AsciiHud<'a> {
    pub fn new(telemetry: &'a dyn TelemetrySource) -> Self {
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
        let hud = HUD::new(telemetry, &symbol_table, 150, aspect_ratio!(5:4));
        Self { hud, screen: [[0u8; 29]; 16] }
    }

    pub fn start_draw<C: Fn(&[[u8; 29]; 16]) -> ()>(&mut self, consumer: C) {
        self.hud.draw(&mut self.screen);
        consumer(&self.screen);
    }
}