use ascii_osd_hud::hud::HUD;
use ascii_osd_hud::symbol::default_symbol_table;
use ascii_osd_hud::telemetry::{Telemetry, TelemetrySource};
use ascii_osd_hud::AspectRatio;

pub struct OSD<'a, S> {
    hud: HUD<'a>,
    buffer: &'a mut [S],
}

pub struct StubTelemetrySource {}

impl TelemetrySource for StubTelemetrySource {
    fn get_telemetry(&self) -> Telemetry {
        Default::default()
    }
}

impl<'a, S: AsMut<[u8]>> OSD<'a, S> {
    pub fn new(telemetry: &'a dyn TelemetrySource, buffer: &'a mut [S]) -> Self {
        let hud = HUD::new(
            telemetry,
            &default_symbol_table(),
            150,
            AspectRatio::Standard,
        );
        Self { hud, buffer }
    }

    pub fn on_timer_timeout(&mut self) {
        self.hud.draw(self.buffer);
    }
}
