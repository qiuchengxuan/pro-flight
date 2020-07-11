#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::Vector3;

use crate::datastructures::measurement::{Altitude, Distance};

use super::measurement::DEGREE_PER_DAG;

const SUB_SECOND: i32 = 10;
const SCALE: i32 = 128;

#[derive(Default, Copy, Clone, Debug)]
pub struct Longitude(i32);

impl Longitude {
    pub fn from_str(string: &str) -> Option<Self> {
        if string.len() != "Dhhh*mm.sss".len() {
            return None;
        }
        if &string[4..5] != "*" || &string[7..8] != "." {
            return None;
        }
        let degree: i32 = match string[1..4].parse().ok() {
            Some(d) => d,
            None => return None,
        };
        let minute: i32 = match string[5..7].parse().ok() {
            Some(m) => m,
            None => return None,
        };
        let sub_second: i32 = match string[8..11].parse().ok() {
            Some(m) => m,
            None => return None,
        };
        let value = (degree * 3600 * SUB_SECOND + minute * 60 * SUB_SECOND + sub_second) * SCALE;
        match &string[0..1] {
            "E" => Some(Longitude(value)),
            "W" => Some(Longitude(-value)),
            _ => None,
        }
    }
}

impl core::ops::Add<Distance<isize>> for Longitude {
    type Output = Self;

    fn add(self, distance: Distance<isize>) -> Self {
        Self(self.0 + distance.0 as i32 * SUB_SECOND * SCALE * 1000 / 30_715)
    }
}

impl core::ops::Sub for Longitude {
    type Output = Distance<isize>;

    fn sub(self, other: Self) -> Distance<isize> {
        Distance(((self.0 - other.0) * 30_715 / 1000 / SCALE / SUB_SECOND) as isize)
    }
}

impl core::fmt::Display for Longitude {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let direction = if self.0 >= 0 { "E" } else { "W" };
        let sub_second = if self.0 >= 0 { self.0 } else { -self.0 } / SCALE;
        let degree = sub_second / SUB_SECOND / 3600;
        let minute = (sub_second / SUB_SECOND / 60) % 60;
        write!(f, "{}{:03}*{:02}.{:03}", direction, degree, minute, sub_second % 600)
    }
}

impl sval::Value for Longitude {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.fmt(format_args!("{}", self))
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Latitude(i32);

impl Latitude {
    pub fn from_str(string: &str) -> Option<Self> {
        if string.len() != "Dhh*mm.sss".len() {
            return None;
        }
        if &string[3..4] != "*" || &string[6..7] != "." {
            return None;
        }
        let degree: i32 = match string[1..3].parse().ok() {
            Some(d) => d,
            None => return None,
        };
        let minute: i32 = match string[4..6].parse().ok() {
            Some(m) => m,
            None => return None,
        };
        let sub_second: i32 = match string[7..10].parse().ok() {
            Some(m) => m,
            None => return None,
        };
        let value = (degree * 3600 * SUB_SECOND + minute * 60 * SUB_SECOND + sub_second) * SCALE;
        match &string[0..1] {
            "N" => Some(Latitude(value)),
            "S" => Some(Latitude(-value)),
            _ => None,
        }
    }
}

impl core::ops::Add<Distance<isize>> for Latitude {
    type Output = Self;

    fn add(self, distance: Distance<isize>) -> Self {
        Self(self.0 + distance.0 as i32 * SUB_SECOND * 100 * SCALE / 30_92)
    }
}

impl core::ops::Sub for Latitude {
    type Output = Distance<isize>;

    fn sub(self, other: Self) -> Distance<isize> {
        Distance(((self.0 - other.0) * 30_92 / SCALE / 100 / SUB_SECOND) as isize)
    }
}

impl core::fmt::Display for Latitude {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let direction = if self.0 >= 0 { "N" } else { "S" };
        let sub_second = if self.0 >= 0 { self.0 } else { -self.0 } / SCALE;
        let degree = sub_second / SUB_SECOND / 3600;
        let minute = (sub_second / SUB_SECOND / 60) % 60;
        write!(f, "{}{:02}*{:02}.{:03}", direction, degree, minute, sub_second % 600)
    }
}

impl sval::Value for Latitude {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.fmt(format_args!("{}", self))
    }
}

#[derive(Copy, Clone, Default, Value)]
pub struct SphericalCoordinate {
    pub rho: Distance<isize>, // radius
    pub theta: i16,           // azimuth angle, [-180, 180]
    pub phi: i8,              // polar angle, [90, 90]
}

#[derive(Default, Copy, Clone)]
pub struct Displacement {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<(f32, f32, f32)> for Displacement {
    fn from(tuple: (f32, f32, f32)) -> Self {
        Self { x: tuple.0, y: tuple.1, z: tuple.2 }
    }
}

impl core::ops::Neg for Displacement {
    type Output = Self;
    fn neg(self) -> Self {
        Self { x: -self.x, y: -self.y, z: -self.z }
    }
}

impl Displacement {
    pub fn azimuth(&self) -> u16 {
        let theta = (self.x.atan2(self.y) * DEGREE_PER_DAG) as i16;
        (if theta > 0 { theta } else { 360 + theta }) as u16
    }

    pub fn into_f32(self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }

    pub fn into_f32_vector(self) -> Vector3<f32> {
        Vector3::new(self.x, self.y, self.z)
    }
}

impl Into<SphericalCoordinate> for Displacement {
    fn into(self) -> SphericalCoordinate {
        let mut square_sum = self.x * self.x + self.y * self.y + self.z * self.z;
        if square_sum <= 0.0 {
            square_sum = 0.0
        }
        let rho = square_sum.sqrt();
        let theta = (self.x.atan2(self.y) * DEGREE_PER_DAG) as i16;
        let phi = if rho.is_normal() { ((self.z / rho).acos() * DEGREE_PER_DAG) as i8 } else { 0 };
        SphericalCoordinate { rho: Distance(rho as isize), theta: theta, phi: phi }
    }
}

impl core::ops::Sub for Displacement {
    type Output = Displacement;

    fn sub(self, other: Self) -> Displacement {
        Displacement { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
}

#[derive(Default, Copy, Clone, Value)]
pub struct Position {
    latitude: Latitude,
    longitude: Longitude,
    altitude: Altitude,
}

impl core::ops::Sub for Position {
    type Output = Displacement;

    fn sub(self, other: Self) -> Displacement {
        let x = self.longitude - other.longitude;
        let y = self.latitude - other.latitude;
        let height = self.altitude - other.altitude;
        Displacement { x: x.0 as f32, y: y.0 as f32, z: (height.0 * 100) as f32 }
    }
}

impl core::ops::Add<Displacement> for Position {
    type Output = Self;

    fn add(self, displacement: Displacement) -> Self {
        let longitude = self.longitude + Distance::<isize>(displacement.x as isize);
        let latitude = self.latitude + Distance::<isize>(displacement.y as isize);
        let altitude = self.altitude + Distance::<isize>(displacement.z as isize);
        Self { latitude, longitude, altitude }
    }
}

mod test {
    #[test]
    fn test_latitude_longitude() {
        use super::{Latitude, Longitude};

        let latitude = Latitude::from_str("N40*19.480").unwrap();
        let longitude = Longitude::from_str("E116*44.540").unwrap();
        let s = format!("{}", latitude);
        assert_eq!("N40*19.480", s);
        let s = format!("{}", longitude);
        assert_eq!("E116*44.540", s);
    }
}
