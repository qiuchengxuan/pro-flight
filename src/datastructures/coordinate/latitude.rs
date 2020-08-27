use crate::datastructures::measurement::distance::Distance;
use crate::datastructures::measurement::unit::{CentiMeter, Meter};

const SUB_SECOND: i32 = 10;
const SCALE: i32 = 128;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Latitude(pub i32);

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

impl PartialEq<i32> for Latitude {
    fn eq(&self, rhs: &i32) -> bool {
        self.0 as i32 == *rhs
    }
}

impl<U: Copy + Into<i32> + Default> core::ops::Add<Distance<i32, U>> for Latitude {
    type Output = Self;

    fn add(self, distance: Distance<i32, U>) -> Self {
        Self(self.0 + distance.to_unit(CentiMeter).value() * SUB_SECOND * 100 * SCALE / 30_92)
    }
}

impl core::ops::Sub for Latitude {
    type Output = Distance<i32, Meter>;

    fn sub(self, other: Self) -> Distance<i32, Meter> {
        let value = (self.0 - other.0) / 100 * 30_92 / SCALE / SUB_SECOND;
        Distance::new(value, Meter)
    }
}

impl core::fmt::Display for Latitude {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let direction = if self.0 >= 0 { "N" } else { "S" };
        let sub_second = if self.0 >= 0 { self.0 } else { -self.0 } / SCALE;
        let degree = sub_second / SUB_SECOND / 3600;
        let minute = (sub_second / SUB_SECOND / 60) % 60;
        write!(f, "{}{:02}°{:02}.{:03}", direction, degree, minute, sub_second % 600)
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

        let latitude = Latitude::from_str("N40°19.480").unwrap();
        assert_eq!("N40°19.480", format!("{}", latitude));

        let distance = latitude - Latitude::from_str("N40°18.480").unwrap();
        assert_eq!("1855m", format!("{}", distance));
    }
}
