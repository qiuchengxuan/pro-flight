use crate::datastructures::measurement::distance::Distance;
use crate::datastructures::measurement::unit::{CentiMeter, Meter};

pub const SUB_SECOND: i32 = 10;
pub const SCALE: i32 = 128;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Longitude(pub i32); // in seconds * SUB_SECOND * SCALE

impl Longitude {
    pub fn from_str(string: &str) -> Option<Self> {
        let positive = match string.chars().next() {
            Some('E') => true,
            Some('W') => false,
            _ => return None,
        };

        let mut split = string[1..].split('°');
        let degree: i32 = match split.next().map(|d| d.parse().ok()).flatten() {
            Some(d) => d,
            None => return None,
        };
        let mut split = match split.next() {
            Some(remain) => remain.split('.'),
            None => return None,
        };
        let minute: i32 = match split.next().map(|m| m.parse().ok()).flatten() {
            Some(m) => m,
            None => return None,
        };
        let sub_second: i32 = match split.next().map(|s| s.parse().ok()).flatten() {
            Some(s) => s,
            None => return None,
        };
        let value = (degree * 3600 * SUB_SECOND + minute * 60 * SUB_SECOND + sub_second) * SCALE;
        Some(Self(if positive { value } else { -value }))
    }
}

impl<U: Copy + Into<i32> + Default> core::ops::Add<Distance<i32, U>> for Longitude {
    type Output = Self;

    fn add(self, delta: Distance<i32, U>) -> Self {
        Self(self.0 + delta.to_unit(CentiMeter).value() * SUB_SECOND * SCALE * 10 / 30_715)
    }
}

impl core::ops::Sub for Longitude {
    type Output = Distance<i32, Meter>;

    fn sub(self, other: Self) -> Distance<i32, Meter> {
        let value = ((self.0 - other.0) as f32 / (SCALE * SUB_SECOND) as f32 * 30.715) as i32;
        Distance::new(value, Meter)
    }
}

impl core::fmt::Display for Longitude {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let direction = if self.0 >= 0 { "E" } else { "W" };
        let sub_second = if self.0 >= 0 { self.0 } else { -self.0 } / SCALE;
        let degree = sub_second / SUB_SECOND / 3600;
        let minute = (sub_second / SUB_SECOND / 60) % 60;
        write!(f, "{}{:03}°{:02}.{:03}", direction, degree, minute, sub_second % 600)
    }
}

impl sval::Value for Longitude {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.fmt(format_args!("{}", self))
    }
}

mod test {
    #[test]
    fn test_longitude() {
        use super::Longitude;

        let longitude = Longitude::from_str("E116°44.540").unwrap();
        assert_eq!("E116°44.540", format!("{}", longitude));

        let distance = longitude - Longitude::from_str("E116°43.540").unwrap();
        assert_eq!("1842m", format!("{}", distance));
    }
}
