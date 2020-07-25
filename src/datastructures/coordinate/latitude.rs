use crate::datastructures::measurement::Distance;

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

impl PartialEq<isize> for Latitude {
    fn eq(&self, rhs: &isize) -> bool {
        self.0 as isize == *rhs
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
    }
}
