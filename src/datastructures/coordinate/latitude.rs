use crate::datastructures::measurement::distance::Distance;
use crate::datastructures::measurement::unit::{CentiMeter, Meter};

pub const SUB_SECOND: i32 = 1000;

const MAX_SECONDS: i32 = 180 * 3600 * SUB_SECOND;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Latitude(pub i32); // in seconds * SUB_SECOND * SCALE

impl Latitude {
    pub fn from_str(string: &str) -> Option<Self> {
        let positive = match string.chars().next() {
            Some('N') => true,
            Some('S') => false,
            _ => return None,
        };

        let mut split = string[1..].split('°');
        let degree: i32 = match split.next().map(|d| d.parse().ok()).flatten() {
            Some(d) => d,
            None => return None,
        };
        let mut split = match split.next() {
            Some(remain) => remain.split('\''),
            None => return None,
        };
        let minute: i32 = match split.next().map(|m| m.parse().ok()).flatten() {
            Some(m) => m,
            None => return None,
        };
        let mut split = match split.next() {
            Some(remain) => remain.split('.'),
            None => return None,
        };
        let second: i32 = match split.next().map(|s| s.parse().ok()).flatten() {
            Some(s) => s,
            None => return None,
        };
        let sub_second: i32 = split.next().map(|s| s.parse().ok()).flatten().unwrap_or(0);
        let value = (degree * 3600 + minute * 60 + second) * SUB_SECOND + sub_second;
        Some(Self(if positive { value } else { -value }))
    }
}

impl PartialEq<i32> for Latitude {
    fn eq(&self, rhs: &i32) -> bool {
        self.0 as i32 == *rhs
    }
}

impl<U: Copy + Into<i32> + Default> core::ops::Add<Distance<i32, U>> for Latitude {
    type Output = Self;

    fn add(self, delta: Distance<i32, U>) -> Self {
        let cm = delta.to_unit(CentiMeter).value() as i64;
        let seconds = self.0 + (cm * SUB_SECOND as i64 / 30_92) as i32;
        if seconds.abs() > MAX_SECONDS {
            return Self(-(seconds % MAX_SECONDS));
        }
        Self(seconds)
    }
}

impl<U: Copy + Into<i32> + Default> core::ops::Sub<Distance<i32, U>> for Latitude {
    type Output = Self;

    fn sub(self, delta: Distance<i32, U>) -> Self {
        let cm = delta.to_unit(CentiMeter).value() as i64;
        let seconds = self.0 - (cm * SUB_SECOND as i64 / 30_92) as i32;
        if seconds.abs() > MAX_SECONDS {
            return Self(-(seconds % MAX_SECONDS));
        }
        Self(seconds)
    }
}

impl core::ops::Sub for Latitude {
    type Output = Distance<i32, Meter>;

    fn sub(self, other: Self) -> Distance<i32, Meter> {
        let delta = (self.0 - other.0) as i64;
        Distance::new((delta * 30_92 / 100 / SUB_SECOND as i64) as i32, Meter)
    }
}

impl core::fmt::Display for Latitude {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let direction = if self.0 >= 0 { "N" } else { "S" };
        let sub_second = self.0.abs();
        let second = sub_second / SUB_SECOND;
        let (degree, minute, second) = (second / 3600, (second / 60) % 60, second % 60);
        let sub_second = sub_second % SUB_SECOND;
        write!(f, "{}{:02}°{:02}'{:02}.{:03}", direction, degree, minute, second, sub_second)
    }
}

impl sval::Value for Latitude {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.fmt(format_args!("{}", self))
    }
}

mod test {
    #[test]
    fn test_latitude() {
        use super::Latitude;
        use crate::datastructures::measurement::distance::Distance;
        use crate::datastructures::measurement::unit::CentiMeter;

        let latitude = Latitude::from_str("N40°19'48").unwrap();
        assert_eq!("N40°19'48.000", format!("{}", latitude));
        assert_eq!("N40°19'47.998", format!("{}", latitude + Distance::new(-7, CentiMeter)));

        let distance = latitude - Latitude::from_str("N40°18'48.000").unwrap();
        assert_eq!("1855m", format!("{}", distance));
    }
}
