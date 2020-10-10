use alloc::boxed::Box;

use ascii_osd_hud::hud::HUD;
use ascii_osd_hud::symbol::default_symbol_table;
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

impl<T: StaticData<TelemetryData>> AsciiHud<T> {
    pub fn new(telemetry: T, fov: u8, pixel_ratio: PixelRatio, aspect_ratio: AspectRatio) -> Self {
        let hud = HUD::new(&default_symbol_table(), fov, pixel_ratio, aspect_ratio);
        Self { hud, telemetry, screen: Box::new([[0u8; 29]; 15]) }
    }

    pub fn draw(&mut self) -> &Screen {
        let data = self.telemetry.read();
        let (status, sensor, nav, misc) =
            (&data.status, &data.sensor, &data.navigation, &data.misc);

        let altitude = status.altitude.to_unit(Feet);
        let height = status.height.to_unit(Feet).value() as i16;
        let delta = (nav.steerpoint.waypoint.position - nav.position).convert(|v| v as f32);
        let vector = misc.quaternion.inverse_transform_vector(&delta.into());
        let transformed: DistanceVector<f32, Meter> = vector.into();
        let coordinate: SphericalCoordinate<Meter> = (transformed * 10.0).into();
        let steerpoint = Steerpoint {
            number: nav.steerpoint.index,
            name: nav.steerpoint.waypoint.name,
            heading: delta.azimuth(),
            coordinate: coordinate.to_unit(NauticalMile).into(),
        };

        let vector = misc.quaternion.inverse_transform_vector(&nav.speed_vector.into());
        let vector: VelocityVector<f32, Meter> = vector.into();
        let speed_vector: SphericalCoordinate<Knot> = vector.to_unit(Knot).into();

        let mut aoa = status.attitude.pitch.wrapping_sub((speed_vector.phi as i16) * 10);
        if aoa > i8::MAX as i16 {
            aoa = i8::MAX as i16;
        } else if aoa < i8::MIN as i16 {
            aoa = i8::MIN as i16;
        }

        let mut note_buffer = [0u8; 30];
        let mut index = 0;
        if let Some(gnss) = sensor.gnss {
            if !gnss.fixed {
                note_buffer[index..index + NO_GPS.len()].copy_from_slice(NO_GPS.as_bytes());
                index += NO_GPS.len();
            }
        }
        let note_left = unsafe { core::str::from_utf8_unchecked(&note_buffer[..index]) };
        let hud_telemetry = Telemetry {
            altitude: altitude.value() as i16,
            aoa: aoa as i8,
            attitude: status.attitude.into(),
            battery: status.battery.percentage(),
            heading: status.heading,
            g_force: status.g_force,
            height: if height > 200 { i16::MIN } else { height },
            notes: Notes { left: note_left, center: "", right: "" },
            rssi: status.rssi as u8,
            unit: Unit::Aviation,
            speed_vector: speed_vector.into(),
            vario: status.vario as i16 / 100 * 100,
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
