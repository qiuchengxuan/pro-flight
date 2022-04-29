use ascii_osd_hud::PixelRatio;

pub type Frame<const W: usize, const H: usize> = [[u8; W]; H];

pub mod nav;
pub mod telemetry;

pub struct OSD {
    nav: nav::NAV,
}

impl OSD {
    pub fn new(pixel_ratio: PixelRatio) -> Self {
        Self { nav: nav::NAV::new(pixel_ratio) }
    }

    pub fn draw<'a, const W: usize, const H: usize>(
        &self,
        frame: &'a mut Frame<W, H>,
    ) -> &'a Frame<W, H> {
        self.nav.draw(frame)
    }
}
