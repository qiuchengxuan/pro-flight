use core::num::FpCategory;

#[allow(unused_imports)] // false warning
use micromath::F32Ext;

pub mod latitude;
pub mod longitude;

pub use latitude::Latitude;
pub use longitude::Longitude;

use crate::types::measurement::{unit, Altitude, Displacement, Distance, ENU};

#[derive(Copy, Clone, Default, Serialize, PartialEq, Debug)]
pub struct SphericalCoordinate<U: Copy> {
    pub rho: Distance<u32, U>, // radius
    pub theta: i16,            // azimuth angle, [-180, 180]
    pub phi: i8,               // polar angle, [-90, 90]
}

impl<U: Copy + Default> From<Displacement<f32, U, ENU>> for SphericalCoordinate<U> {
    fn from(vector: Displacement<f32, U, ENU>) -> SphericalCoordinate<U> {
        let rho = vector.scalar();
        if rho.raw.classify() == FpCategory::Zero {
            return Self::default();
        }
        let (x, y, z) = vector.into();
        let theta = x.atan2(y).to_degrees() as i16;
        let phi = if z >= 0.0 {
            90 - (z / rho.raw).acos().to_degrees() as i8
        } else {
            (-z / rho.raw).acos().to_degrees() as i8 - 90
        };
        SphericalCoordinate { rho: rho.t(|v| v as u32), theta, phi }
    }
}

impl<U: Copy + Default + Into<u32>> SphericalCoordinate<U> {
    pub fn u<V>(self, unit: V) -> SphericalCoordinate<V>
    where
        V: Copy + Default + Into<u32> + unit::Distance,
    {
        SphericalCoordinate { rho: self.rho.u(unit), theta: self.theta, phi: self.phi }
    }
}

#[derive(Default, Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Position {
    pub latitude: Latitude,
    pub longitude: Longitude,
    pub altitude: Altitude,
}

impl core::ops::Sub for Position {
    type Output = Displacement<i32, unit::Meter, ENU>;

    fn sub(self, other: Self) -> Self::Output {
        let x = self.longitude - other.longitude;
        let y = self.latitude - other.latitude;
        let height = self.altitude - other.altitude;
        Displacement::new(x.raw, y.raw, height.u(unit::Meter).raw, unit::Meter, ENU)
    }
}

impl<U: Copy + Default + Into<i32>> core::ops::Add<Displacement<i32, U, ENU>> for Position {
    type Output = Self;

    fn add(self, displacement: Displacement<i32, U, ENU>) -> Self {
        let longitude = self.longitude + displacement.x();
        let latitude = self.latitude + displacement.y();
        let altitude = self.altitude + displacement.z().u(unit::CentiMeter);
        Self { latitude, longitude, altitude }
    }
}

impl<U: Copy + Default + Into<i32>> core::ops::Sub<Displacement<i32, U, ENU>> for Position {
    type Output = Self;

    fn sub(self, displacement: Displacement<i32, U, ENU>) -> Self {
        let longitude = self.longitude - displacement.x();
        let latitude = self.latitude - displacement.y();
        let altitude = self.altitude - displacement.z().u(unit::CentiMeter);
        Self { latitude, longitude, altitude }
    }
}

mod test {
    #[test]
    fn test_spherical_coordinate() {
        use crate::types::measurement::{unit::Meter, Displacement, Distance, ENU};

        use super::SphericalCoordinate;

        let vector = Displacement::new(0.0, 0.0, 0.0, Meter, ENU);
        let coordinate: SphericalCoordinate<Meter> = vector.into();
        let expected = SphericalCoordinate { rho: Distance::default(), theta: 0, phi: 0 };
        assert_eq!(coordinate, expected);

        let vector = Displacement::new(60.0, 100.0, 0.0, Meter, ENU);
        let coordinate: SphericalCoordinate<Meter> = vector.into();
        let expected = SphericalCoordinate { rho: Distance::new(116, Meter), theta: 30, phi: 0 };
        assert_eq!(coordinate, expected);

        let vector = Displacement::new(-60.0, 100.0, 0.0, Meter, ENU);
        let coordinate: SphericalCoordinate<Meter> = vector.into();
        assert_eq!(coordinate, SphericalCoordinate {
            rho: Distance::new(116, Meter),
            theta: -30,
            phi: 0
        });

        let vector = Displacement::new(0.0, 100.0, 60.0, Meter, ENU);
        let coordinate: SphericalCoordinate<Meter> = vector.into();
        assert_eq!(coordinate, SphericalCoordinate {
            rho: Distance::new(116, Meter),
            theta: 0,
            phi: 31
        });

        let vector = Displacement::new(0.0, 100.0, -60.0, Meter, ENU);
        let coordinate: SphericalCoordinate<Meter> = vector.into();
        assert_eq!(coordinate, SphericalCoordinate {
            rho: Distance::new(116, Meter),
            theta: 0,
            phi: -31
        });
    }
}
