pub mod latitude;
pub mod longitude;

#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::Vector3;

pub use latitude::Latitude;
pub use longitude::Longitude;

use crate::datastructures::measurement::distance::Distance;
use crate::datastructures::measurement::unit::{CentiMeter, Meter};
use crate::datastructures::measurement::Altitude;

use super::measurement::DEGREE_PER_DAG;

#[derive(Copy, Clone, Default, Value, PartialEq, Debug)]
pub struct SphericalCoordinate {
    pub rho: Distance<u32, CentiMeter>, // radius
    pub theta: i16,                     // azimuth angle, [-180, 180]
    pub phi: i8,                        // polar angle, [-90, 90]
}

#[derive(Default, Copy, Clone, PartialEq, Debug, Value)]
pub struct Displacement {
    pub x: Distance<i32, CentiMeter>,
    pub y: Distance<i32, CentiMeter>,
    pub z: Distance<i32, CentiMeter>,
}

impl From<(f32, f32, f32)> for Displacement {
    fn from(tuple: (f32, f32, f32)) -> Self {
        Self {
            x: Distance::new(tuple.0, Meter).to_unit(CentiMeter).convert(|v| v as i32),
            y: Distance::new(tuple.1, Meter).to_unit(CentiMeter).convert(|v| v as i32),
            z: Distance::new(tuple.2, Meter).to_unit(CentiMeter).convert(|v| v as i32),
        }
    }
}

impl Displacement {
    pub fn azimuth(&self) -> u16 {
        let theta = ((self.x.value() as f32).atan2(self.y.value() as f32) * DEGREE_PER_DAG) as i16;
        (if theta > 0 { theta } else { 360 + theta }) as u16
    }

    pub fn into_f32(self) -> (f32, f32, f32) {
        (
            self.x.to_unit(Meter).value() as f32,
            self.y.to_unit(Meter).value() as f32,
            self.z.to_unit(Meter).value() as f32,
        )
    }

    pub fn into_f32_vector(self) -> Vector3<f32> {
        let (x, y, z) = self.into_f32();
        Vector3::new(x, y, z)
    }
}

impl Into<SphericalCoordinate> for Displacement {
    fn into(self) -> SphericalCoordinate {
        if self.x.value() + self.y.value() + self.z.value() == 0 {
            return SphericalCoordinate::default();
        }
        let (x, y, z) = (self.x.value() as f32, self.y.value() as f32, self.z.value() as f32);
        let rho = (x * x + y * y + z * z).sqrt();
        let theta = (x.atan2(y) * DEGREE_PER_DAG) as i16;
        let phi = if z >= 0.0 {
            90 - ((z / rho).acos() * DEGREE_PER_DAG) as i8
        } else {
            ((-z / rho).acos() * DEGREE_PER_DAG) as i8 - 90
        };
        SphericalCoordinate { rho: Distance::new(rho as u32, CentiMeter {}), theta, phi }
    }
}

impl core::ops::Sub for Displacement {
    type Output = Displacement;

    fn sub(self, other: Self) -> Displacement {
        Displacement { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
}

#[derive(Default, Copy, Clone, Value, PartialEq, Debug)]
pub struct Position {
    pub latitude: Latitude,
    pub longitude: Longitude,
    pub altitude: Altitude,
}

impl core::ops::Sub for Position {
    type Output = Displacement;

    fn sub(self, other: Self) -> Displacement {
        let x = self.longitude - other.longitude;
        let y = self.latitude - other.latitude;
        let height = self.altitude - other.altitude;
        Displacement { x: x, y: y, z: height.into() }
    }
}

impl core::ops::Add<Displacement> for Position {
    type Output = Self;

    fn add(self, displacement: Displacement) -> Self {
        let longitude = self.longitude + displacement.x;
        let latitude = self.latitude + displacement.y;
        let altitude = self.altitude + displacement.z;
        Self { latitude, longitude, altitude }
    }
}

mod test {
    #[test]
    fn test_spherical_coordinate() {
        use crate::datastructures::measurement::distance::{CentiMeter, Distance};

        use super::{Displacement, SphericalCoordinate};

        let displacement = Displacement::default();
        let coordinate: SphericalCoordinate = displacement.into();
        let expected = SphericalCoordinate { rho: Distance::new(0, CentiMeter), theta: 0, phi: 0 };
        assert_eq!(coordinate, expected);

        let displacement = Displacement {
            x: Distance::new(60, CentiMeter),
            y: Distance::new(100, CentiMeter),
            z: Distance::new(0, CentiMeter),
        };
        let coordinate: SphericalCoordinate = displacement.into();
        let expected =
            SphericalCoordinate { rho: Distance::new(116, CentiMeter), theta: 30, phi: 0 };
        assert_eq!(coordinate, expected);

        let displacement = Displacement {
            x: Distance::new(-60, CentiMeter),
            y: Distance::new(100, CentiMeter),
            z: Distance::new(0, CentiMeter),
        };
        let coordinate: SphericalCoordinate = displacement.into();
        assert_eq!(
            coordinate,
            SphericalCoordinate { rho: Distance::new(116, CentiMeter), theta: -30, phi: 0 }
        );

        let displacement = Displacement {
            x: Distance::new(0, CentiMeter),
            y: Distance::new(100, CentiMeter),
            z: Distance::new(60, CentiMeter),
        };
        let coordinate: SphericalCoordinate = displacement.into();
        assert_eq!(
            coordinate,
            SphericalCoordinate { rho: Distance::new(116, CentiMeter), theta: 0, phi: 31 }
        );

        let displacement = Displacement {
            x: Distance::new(0, CentiMeter),
            y: Distance::new(100, CentiMeter),
            z: Distance::new(-60, CentiMeter),
        };
        let coordinate: SphericalCoordinate = displacement.into();
        assert_eq!(
            coordinate,
            SphericalCoordinate { rho: Distance::new(116, CentiMeter), theta: 0, phi: -31 }
        );
    }
}
