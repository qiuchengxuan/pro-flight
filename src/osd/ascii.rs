use alloc::boxed::Box;
use core::time;

use ascii_osd_hud::{
    hud::HUD,
    symbol::default_symbol_table,
    telemetry::{self as hud, Notes, Steerpoint, Telemetry, Unit},
    AspectRatio, PixelRatio,
};

use crate::{
    collection, config, datastore,
    types::{
        coordinate::SphericalCoordinate,
        measurement::{
            unit::{FTmin, Feet, Knot, Meter, NauticalMile},
            Attitude, Displacement, VelocityVector, ENU,
        },
        Ratio,
    },
};

type Screen<const W: usize, const H: usize> = [[u8; W]; H];

pub type ScreenConsumer<const W: usize, const H: usize> = fn(&Screen<W, H>);

const INS_DEGRADE: &str = "INS DEGD";

pub struct OSD<const W: usize, const H: usize> {
    interval: time::Duration,
    hud: HUD,
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
    hud::Attitude { roll: attitude.roll as i16, pitch: attitude.pitch as i8 }
}

fn hud_coordinate<U: Copy>(c: SphericalCoordinate<U>) -> hud::SphericalCoordinate {
    hud::SphericalCoordinate { rho: c.rho.raw as u16, theta: c.theta, phi: c.phi }
}

impl<const W: usize, const H: usize> OSD<W, H> {
    pub fn new(interval: time::Duration, pixel_ratio: PixelRatio) -> Self {
        let cfg = &config::get().osd;
        let hud = HUD::new(&default_symbol_table(), cfg.fov, pixel_ratio, cfg.aspect_ratio.into());
        Self { interval, hud, screen: Box::new([[0u8; W]; H]) }
    }

    pub fn draw(&mut self) -> &Screen<W, H> {
        let collector = collection::Collector::new(datastore::acquire());
        let data = collector.collect(Some(self.interval));

        let height = data.ins.displacement.z().u(Feet).raw as i16;
        let delta = data.ins.displacement;
        let vector = data.imu.quaternion.inverse_transform_vector(&delta.raw);
        let transformed = Displacement::from(vector, Meter, ENU);
        let coordinate: SphericalCoordinate<Meter> = (transformed * 10.0).into();
        let steerpoint = Steerpoint {
            number: 0,
            name: "HOME",
            heading: delta.azimuth(),
            coordinate: hud_coordinate(coordinate.u(NauticalMile)),
        };

        let vector = data.imu.quaternion.inverse_transform_vector(&data.ins.velocity_vector.raw);
        let vector = VelocityVector::from(vector.into(), Meter, ENU);
        let speed_vector: SphericalCoordinate<Knot> = vector.u(Knot).into();

        let pitch = (data.imu.attitude.pitch * 10.0) as i16;
        let mut aoa = pitch.wrapping_sub((speed_vector.phi as i16) * 10);
        if aoa > i8::MAX as i16 {
            aoa = i8::MAX as i16;
        } else if aoa < i8::MIN as i16 {
            aoa = i8::MIN as i16;
        }

        let mut note_buffer = [0u8; W];
        let mut index = 0;
        if let Some(gnss) = data.gnss {
            if gnss.fixed.is_none() {
                let slice = &mut note_buffer[index..index + INS_DEGRADE.len()];
                slice.copy_from_slice(INS_DEGRADE.as_bytes());
                index += INS_DEGRADE.len();
            }
        }
        let note_left = unsafe { core::str::from_utf8_unchecked(&note_buffer[..index]) };
        let hud_telemetry = Telemetry {
            altitude: data.ins.position.altitude.0.u(Feet).raw as i16,
            aoa: aoa as i8,
            attitude: hud_attitude(data.imu.attitude),
            heading: data.imu.attitude.yaw as u16,
            g_force: data.imu.acceleration.g_force(),
            height: if height > 200 { i16::MIN } else { height },
            notes: Notes { left: note_left, center: "", right: "" },
            battery: data.voltage.soc(),
            rssi: data.control.rssi as u8,
            unit: Unit::Aviation,
            speed_vector: hud_coordinate(speed_vector),
            vario: data.ins.velocity_vector.z().u(FTmin).raw as i16 / 100 * 100,
            steerpoint,
        };
        self.hud.draw(&hud_telemetry, self.screen.as_mut());
        &self.screen
    }
}

mod test {
    #[test]
    fn test_speed_vector() {
        use ascii_osd_hud::telemetry as hud;

        use crate::types::{
            coordinate::SphericalCoordinate,
            measurement::{
                unit::{Knot, Meter},
                VelocityVector, ENU,
            },
        };

        let vector = VelocityVector::new(10.0, 10.0, 10.0, Meter, ENU);
        let speed_vector: SphericalCoordinate<Knot> = vector.u(Knot).into();
        let vector: hud::SphericalCoordinate = super::hud_coordinate(speed_vector);
        assert_eq!(vector, hud::SphericalCoordinate { rho: 33, theta: 45, phi: 36 });
    }
}
