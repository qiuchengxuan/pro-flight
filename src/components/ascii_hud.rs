use alloc::boxed::Box;

use ascii_osd_hud::hud::HUD;
use ascii_osd_hud::symbol::default_symbol_table;
use ascii_osd_hud::telemetry::{self as hud, Notes, Steerpoint, Telemetry, Unit};
use ascii_osd_hud::{AspectRatio, PixelRatio};

use crate::components::flight_data_hub::FlightDataReader;
use crate::config;
use crate::datastructures::{
    coordinate::SphericalCoordinate,
    flight::aviation::Attitude,
    measurement::{
        displacement::DistanceVector,
        unit::{Feet, Knot, Meter, NauticalMile},
        VelocityVector,
    },
    Ratio,
};

type Screen<const W: usize, const H: usize> = [[u8; W]; H];

pub type ScreenConsumer<const W: usize, const H: usize> = fn(&Screen<W, H>);

const NO_GPS: &str = "NO GPS";

pub struct AsciiHud<'a, const W: usize, const H: usize> {
    hud: HUD,
    reader: FlightDataReader<'a>,
    screen: Box<[[u8; W]; H]>,
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

fn hud_attitude(attitude: Attitude) -> hud::Attitude {
    hud::Attitude { roll: attitude.roll / 10, pitch: (attitude.pitch / 10) as i8 }
}

fn hud_coordinate<U: Copy>(c: SphericalCoordinate<U>) -> hud::SphericalCoordinate {
    hud::SphericalCoordinate { rho: c.rho.value() as u16, theta: c.theta, phi: c.phi }
}

impl<'a, const W: usize, const H: usize> AsciiHud<'a, W, H> {
    pub fn new(reader: FlightDataReader<'a>, pixel_ratio: PixelRatio) -> Self {
        let cfg = &config::get().osd;
        let hud = HUD::new(&default_symbol_table(), cfg.fov, pixel_ratio, cfg.aspect_ratio.into());
        Self { hud, reader, screen: Box::new([[0u8; W]; H]) }
    }

    pub fn draw(&mut self) -> &Screen<W, H> {
        let data = self.reader.read();

        let height = data.aviation.height.to_unit(Feet).value() as i16;
        let nav = &data.navigation;
        let delta = (nav.steerpoint.waypoint.position - nav.position).convert(|v| v as f32);

        let vector = data.misc.quaternion.inverse_transform_vector(&delta.into());
        let transformed: DistanceVector<f32, Meter> = vector.into();
        let coordinate: SphericalCoordinate<Meter> = (transformed * 10.0).into();
        let steerpoint = Steerpoint {
            number: nav.steerpoint.index,
            name: nav.steerpoint.waypoint.name,
            heading: delta.azimuth(),
            coordinate: hud_coordinate(coordinate.to_unit(NauticalMile)),
        };

        let vector = data.misc.quaternion.inverse_transform_vector(&nav.speed_vector.into());
        let vector: VelocityVector<f32, Meter> = vector.into();
        let speed_vector: SphericalCoordinate<Knot> = vector.to_unit(Knot).into();

        let mut aoa = data.aviation.attitude.pitch.wrapping_sub((speed_vector.phi as i16) * 10);
        if aoa > i8::MAX as i16 {
            aoa = i8::MAX as i16;
        } else if aoa < i8::MIN as i16 {
            aoa = i8::MIN as i16;
        }

        let mut note_buffer = [0u8; W];
        let mut index = 0;
        if let Some(gnss) = data.sensor.gnss {
            if !gnss.fixed {
                note_buffer[index..index + NO_GPS.len()].copy_from_slice(NO_GPS.as_bytes());
                index += NO_GPS.len();
            }
        }
        let note_left = unsafe { core::str::from_utf8_unchecked(&note_buffer[..index]) };
        let hud_telemetry = Telemetry {
            altitude: data.aviation.altitude.to_unit(Feet).value() as i16,
            aoa: aoa as i8,
            attitude: hud_attitude(data.aviation.attitude),
            heading: data.aviation.heading,
            g_force: data.aviation.g_force,
            height: if height > 200 { i16::MIN } else { height },
            notes: Notes { left: note_left, center: "", right: "" },
            battery: data.misc.battery.percentage(),
            rssi: data.misc.rssi as u8,
            unit: Unit::Aviation,
            speed_vector: hud_coordinate(speed_vector),
            vario: data.aviation.vario as i16 / 100 * 100,
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
        let vector: hud::SphericalCoordinate = super::hud_coordinate(speed_vector);
        assert_eq!(vector, hud::SphericalCoordinate { rho: 33, theta: 45, phi: 36 });
    }
}
