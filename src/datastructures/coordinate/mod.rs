use core::num::FpCategory;

#[allow(unused_imports)] // false warning
use micromath::F32Ext;

pub mod latitude;
pub mod longitude;

pub use latitude::Latitude;
pub use longitude::Longitude;

use crate::datastructures::measurement::displacement::DistanceVector;
use crate::datastructures::measurement::distance::Distance;
use crate::datastructures::measurement::euler::DEGREE_PER_DAG;
use crate::datastructures::measurement::unit::CentiMeter;
use crate::datastructures::measurement::Altitude;

#[derive(Copy, Clone, Default, Value, PartialEq, Debug)]
pub struct SphericalCoordinate<U: Copy> {
    pub rho: Distance<u32, U>, // radius
    pub theta: i16,            // azimuth angle, [-180, 180]
    pub phi: i8,               // polar angle, [-90, 90]
}

impl<U: Copy + Default> From<DistanceVector<f32, U>> for SphericalCoordinate<U> {
    fn from(vector: DistanceVector<f32, U>) -> SphericalCoordinate<U> {
        let rho = vector.distance();
        if rho.value().classify() == FpCategory::Zero {
            return SphericalCoordinate { rho: rho.convert(|v| v as u32), theta: 0, phi: 0 };
        }
        let (x, y, z) = vector.into();
        let theta = (x.atan2(y) * DEGREE_PER_DAG) as i16;
        let phi = if z >= 0.0 {
            90 - ((z / rho.value()).acos() * DEGREE_PER_DAG) as i8
        } else {
            ((-z / rho.value()).acos() * DEGREE_PER_DAG) as i8 - 90
        };
        SphericalCoordinate { rho: rho.convert(|v| v as u32), theta, phi }
    }
}

pub type Displacement = DistanceVector<i32, CentiMeter>;

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
        use crate::datastructures::measurement::displacement::DistanceVector;
        use crate::datastructures::measurement::distance::Distance;
        use crate::datastructures::measurement::unit::Meter;

        use super::SphericalCoordinate;

        let vector = DistanceVector::default();
        let coordinate: SphericalCoordinate<Meter> = vector.into();
        let expected = SphericalCoordinate { rho: Distance::default(), theta: 0, phi: 0 };
        assert_eq!(coordinate, expected);

        let vector = DistanceVector::new(60.0, 100.0, 0.0, Meter);
        let coordinate: SphericalCoordinate<Meter> = vector.into();
        let expected = SphericalCoordinate { rho: Distance::new(116, Meter), theta: 30, phi: 0 };
        assert_eq!(coordinate, expected);

        let vector = DistanceVector::new(-60.0, 100.0, 0.0, Meter);
        let coordinate: SphericalCoordinate<Meter> = vector.into();
        assert_eq!(
            coordinate,
            SphericalCoordinate { rho: Distance::new(116, Meter), theta: -30, phi: 0 }
        );

        let vector = DistanceVector::new(0.0, 100.0, 60.0, Meter);
        let coordinate: SphericalCoordinate<Meter> = vector.into();
        assert_eq!(
            coordinate,
            SphericalCoordinate { rho: Distance::new(116, Meter), theta: 0, phi: 31 }
        );

        let vector = DistanceVector::new(0.0, 100.0, -60.0, Meter);
        let coordinate: SphericalCoordinate<Meter> = vector.into();
        assert_eq!(
            coordinate,
            SphericalCoordinate { rho: Distance::new(116, Meter), theta: 0, phi: -31 }
        );
    }
}
