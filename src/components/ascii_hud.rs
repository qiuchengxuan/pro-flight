use alloc::boxed::Box;

use ascii_osd_hud::hud::HUD;
use ascii_osd_hud::symbol::{Symbol, SymbolTable};
use ascii_osd_hud::telemetry::{Notes, Steerpoint, Telemetry, Unit};
use ascii_osd_hud::{AspectRatio, PixelRatio};

use crate::components::telemetry::TelemetryData;
use crate::datastructures::coordinate::{Displacement, SphericalCoordinate};
use crate::datastructures::data_source::StaticData;
use crate::datastructures::gnss::FixType;
use crate::datastructures::measurement::unit::Feet;
use crate::datastructures::Ratio;

type Screen = [[u8; 29]; 15];

pub type ScreenConsumer = fn(&Screen);

const NO_GPS: &str = "NO GPS";

pub struct AsciiHud<T> {
    hud: HUD,
    telemetry: T,
    screen: Box<[[u8; 29]; 15]>,
}

impl From<Ratio> for AspectRatio {
    fn from(ratio: Ratio) -> AspectRatio {
        AspectRatio(ratio.0, ratio.1)
    }
}

impl From<Ratio> for PixelRatio {
    fn from(ratio: Ratio) -> PixelRatio {
        PixelRatio(ratio.0, ratio.1)
    }
}

fn round_up(value: i16) -> i16 {
    (value + 5) / 10 * 10
}

impl<T: StaticData<TelemetryData>> AsciiHud<T> {
    pub fn new(telemetry: T, fov: u8, pixel_ratio: PixelRatio, aspect_ratio: AspectRatio) -> Self {
        let symbol_table: SymbolTable = enum_map! {
            Symbol::Antenna => 1,
            Symbol::Battery => 144,
            Symbol::Degree => 168,
            Symbol::VeclocityVector => 126,
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
        let hud = HUD::new(&symbol_table, fov, pixel_ratio, aspect_ratio);
        Self { hud, telemetry, screen: Box::new([[0u8; 29]; 15]) }
    }

    pub fn draw(&mut self) -> &Screen {
        let data = self.telemetry.read();

        let altitude = data.altitude.to_unit(Feet);
        let height = data.height.to_unit(Feet);
        let delta = data.steerpoint.waypoint.position - data.position;
        let vector = data.raw.quaternion.inverse_transform_vector(&delta.into_f32_vector());
        let transformed: Displacement = (vector[0], vector[1], vector[2]).into();
        let coordinate: SphericalCoordinate = transformed.into();
        let steerpoint = Steerpoint {
            number: data.steerpoint.index,
            name: data.steerpoint.waypoint.name,
            heading: delta.azimuth(),
            coordinate: coordinate.into(),
        };

        let mut note_buffer = [0u8; 30];
        let mut index = 0;
        if let Some(fix_type) = data.raw.fix_type {
            if fix_type == FixType::NoFix {
                note_buffer[index..index + NO_GPS.len()].copy_from_slice(NO_GPS.as_bytes());
                index += NO_GPS.len();
            }
        }
        let note_left = unsafe { core::str::from_utf8_unchecked(&note_buffer[..index]) };
        let hud_telemetry = Telemetry {
            altitude: round_up(altitude.value() as i16),
            attitude: data.attitude.into(),
            battery: data.battery.percentage(),
            heading: data.heading,
            g_force: data.g_force,
            height: height.value() as i16,
            unit: Unit::Aviation,
            vario: data.velocity.value() / 100 * 100,
            steerpoint: steerpoint,
            notes: Notes { left: note_left, center: "", right: "" },
            ..Default::default()
        };
        self.hud.draw(&hud_telemetry, self.screen.as_mut());
        &self.screen
    }
}
