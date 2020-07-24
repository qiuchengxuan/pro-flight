pub mod latitude;
pub mod longitude;

#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::Vector3;

pub use latitude::Latitude;
pub use longitude::Longitude;

use crate::datastructures::measurement::{Altitude, Distance, DistanceUnit};

use super::measurement::DEGREE_PER_DAG;

#[derive(Copy, Clone, Default, Value, PartialEq, Debug)]
pub struct SphericalCoordinate {
    pub rho: Distance<isize>, // radius
    pub theta: i16,           // azimuth angle, [-180, 180]
    pub phi: i8,              // polar angle, [-90, 90]
}

// assuming unit of centimeter
#[derive(Default, Copy, Clone, PartialEq, Debug, Value)]
pub struct Displacement {
    pub x: Distance<isize>,
    pub y: Distance<isize>,
    pub z: Distance<isize>,
}

impl From<(f32, f32, f32)> for Displacement {
    fn from(tuple: (f32, f32, f32)) -> Self {
        let unit = DistanceUnit::Meter as isize as f32;
        let x = Distance((tuple.0 * unit) as isize);
        let y = Distance((tuple.1 * unit) as isize);
        let z = Distance((tuple.2 * unit) as isize);
        Self { x, y, z }
    }
}

impl Displacement {
    pub fn azimuth(&self) -> u16 {
        let theta = ((self.x.0 as f32).atan2(self.y.0 as f32) * DEGREE_PER_DAG) as i16;
        (if theta > 0 { theta } else { 360 + theta }) as u16
    }

    pub fn into_f32(self) -> (f32, f32, f32) {
        (self.x.into(), self.y.into(), self.z.into())
    }

    pub fn into_f32_vector(self) -> Vector3<f32> {
        Vector3::new(self.x.into(), self.y.into(), self.z.into())
    }
}

impl Into<SphericalCoordinate> for Displacement {
    fn into(self) -> SphericalCoordinate {
        if self.x.0 + self.y.0 + self.z.0 == 0 {
            return SphericalCoordinate { rho: Distance(0), theta: 0, phi: 0 };
        }
        let (x, y, z) = (self.x.0 as f32, self.y.0 as f32, self.z.0 as f32);
        let rho = (x * x + y * y + z * z).sqrt();
        let theta = (x.atan2(y) * DEGREE_PER_DAG) as i16;
        let phi = if z >= 0.0 {
            90 - ((z / rho).acos() * DEGREE_PER_DAG) as i8
        } else {
            ((-z / rho).acos() * DEGREE_PER_DAG) as i8 - 90
        };
        SphericalCoordinate { rho: Distance(rho as isize), theta, phi }
    }
}

impl core::ops::Sub for Displacement {
    type Output = Displacement;

    fn sub(self, other: Self) -> Displacement {
        Displacement { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
}

#[derive(Default, Copy, Clone, Value, PartialEq)]
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
        Displacement {
            x: x,
            y: y,
            z: Distance(height.convert(DistanceUnit::Meter, DistanceUnit::CentiMeter, 1)),
        }
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
        use crate::datastructures::measurement::Distance;

        use super::{Displacement, SphericalCoordinate};

        let displacement = Displacement { x: Distance(0), y: Distance(0), z: Distance(0) };
        let coordinate: SphericalCoordinate = displacement.into();
        assert_eq!(coordinate, SphericalCoordinate { rho: Distance(0), theta: 0, phi: 0 });

        let displacement = Displacement { x: Distance(60), y: Distance(100), z: Distance(0) };
        let coordinate: SphericalCoordinate = displacement.into();
        assert_eq!(coordinate, SphericalCoordinate { rho: Distance(116), theta: 30, phi: 0 });

        let displacement = Displacement { x: Distance(-60), y: Distance(100), z: Distance(0) };
        let coordinate: SphericalCoordinate = displacement.into();
        assert_eq!(coordinate, SphericalCoordinate { rho: Distance(116), theta: -30, phi: 0 });

        let displacement = Displacement { x: Distance(0), y: Distance(100), z: Distance(60) };
        let coordinate: SphericalCoordinate = displacement.into();
        assert_eq!(coordinate, SphericalCoordinate { rho: Distance(116), theta: 0, phi: 31 });

        let displacement = Displacement { x: Distance(0), y: Distance(100), z: Distance(-60) };
        let coordinate: SphericalCoordinate = displacement.into();
        assert_eq!(coordinate, SphericalCoordinate { rho: Distance(116), theta: 0, phi: -31 });
    }
}
