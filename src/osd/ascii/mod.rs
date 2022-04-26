use ascii_osd_hud::PixelRatio;
use fugit::NanosDurationU64 as Duration;

use crate::{config::inputs::command, datastore};

pub type Frame<const W: usize, const H: usize> = [[u8; W]; H];

pub mod nav;
pub mod telemetry;

pub struct OSD {
    nav: nav::NAV,
    telemetry: telemetry::Telemetry,
}

impl OSD {
    pub fn new(pixel_ratio: PixelRatio) -> Self {
        Self { nav: nav::NAV::new(pixel_ratio), telemetry: telemetry::Telemetry }
    }

    pub fn draw<'a, const W: usize, const H: usize>(
        &self,
        frame: &'a mut Frame<W, H>,
    ) -> &'a Frame<W, H> {
        let ds = datastore::acquire();
        let mut mode = command::Mode::NAV;
        if let Some(control) = ds.read_control_within(Duration::millis(100)) {
            for command in control.commands.iter() {
                match command {
                    command::Id::Mode(m) => mode = *m,
                }
            }
        }
        match mode {
            command::Mode::NAV => self.nav.draw(frame),
            command::Mode::Telemetry => self.telemetry.draw(frame),
        }
    }
}
