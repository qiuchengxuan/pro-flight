use alloc::boxed::Box;

use ascii_osd_hud::hud::HUD;
use ascii_osd_hud::symbol::{Symbol, SymbolTable};
use ascii_osd_hud::telemetry::{Notes, Steerpoint, Telemetry, Unit};
use ascii_osd_hud::{AspectRatio, PixelRatio};

use crate::components::telemetry::TelemetryData;
use crate::datastructures::coordinate::SphericalCoordinate;
use crate::datastructures::data_source::StaticData;
use crate::datastructures::measurement::displacement::DistanceVector;
use crate::datastructures::measurement::unit::{Feet, Knot, Meter, NauticalMile};
use crate::datastructures::measurement::VelocityVector;
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
        let (basic, misc, raw) = (&data.basic, &data.misc, &data.raw);

        let altitude = basic.altitude.to_unit(Feet);
        let height = basic.height.to_unit(Feet);
        let delta = (misc.steerpoint.waypoint.position - misc.position).convert(|v| v as f32);
        let vector = raw.quaternion.inverse_transform_vector(&delta.into());
        let transformed: DistanceVector<f32, Meter> = vector.into();
        let coordinate: SphericalCoordinate<Meter> = (transformed * 10.0).into();
        let steerpoint = Steerpoint {
            number: misc.steerpoint.index,
            name: misc.steerpoint.waypoint.name,
            heading: delta.azimuth(),
            coordinate: coordinate.to_unit(NauticalMile).into(),
        };

        let vector = raw.quaternion.inverse_transform_vector(&raw.speed_vector.into());
        let vector: VelocityVector<f32, Meter> = vector.into();
        let speed_vector: SphericalCoordinate<Knot> = vector.to_unit(Knot).into();

        let mut aoa = basic.attitude.pitch.wrapping_sub((speed_vector.phi as i16) * 10);
        if aoa > i8::MAX as i16 {
            aoa = i8::MAX as i16;
        } else if aoa < i8::MIN as i16 {
            aoa = i8::MIN as i16;
        }

        let mut note_buffer = [0u8; 30];
        let mut index = 0;
        if let Some(gnss) = raw.gnss {
            if !gnss.fixed {
                note_buffer[index..index + NO_GPS.len()].copy_from_slice(NO_GPS.as_bytes());
                index += NO_GPS.len();
            }
        }
        let note_left = unsafe { core::str::from_utf8_unchecked(&note_buffer[..index]) };
        let hud_telemetry = Telemetry {
            altitude: round_up(altitude.value() as i16),
            aoa: aoa as i8,
            attitude: basic.attitude.into(),
            battery: misc.battery.percentage(),
            heading: basic.heading,
            g_force: basic.g_force,
            height: height.value() as i16,
            notes: Notes { left: note_left, center: "", right: "" },
            rssi: misc.rssi as u8,
            unit: Unit::Aviation,
            speed_vector: speed_vector.into(),
            vario: basic.vario as i16 / 100 * 100,
            steerpoint: steerpoint,
        };
        self.hud.draw(&hud_telemetry, self.screen.as_mut());
        &self.screen
    }
}

mod test {
    #[test]
    fn test_speed_vector() {
        use ascii_osd_hud::telemetry as hud;

        use crate::datastructures::coordinate::SphericalCoordinate;
        use crate::datastructures::measurement::unit::{Knot, Meter};
        use crate::datastructures::measurement::VelocityVector;

        let vector: VelocityVector<f32, Meter> = VelocityVector::new(10.0, 10.0, 10.0, Meter);
        let speed_vector: SphericalCoordinate<Knot> = vector.to_unit(Knot).into();
        let vector: hud::SphericalCoordinate = speed_vector.into();
        assert_eq!(vector, hud::SphericalCoordinate { rho: 33, theta: 45, phi: 36 });
    }
}
