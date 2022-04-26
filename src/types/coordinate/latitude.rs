use core::{
    convert::TryFrom,
    fmt::{self, Write},
};

use heapless::String;

use crate::types::measurement::{
    unit::{CentiMeter, Meter},
    Distance,
};

pub const SUB_SECOND: i32 = 1000;

const MAX_SECONDS: i32 = 180 * 3600 * SUB_SECOND;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Latitude<const D: char>(pub i32); // in seconds * SUB_SECOND * SCALE

impl<const D: char> Latitude<D> {
    pub fn into<const T: char>(self) -> Latitude<T> {
        Latitude(self.0)
    }
}

impl TryFrom<&str> for Latitude<'°'> {
    type Error = ();

    fn try_from(string: &str) -> Result<Self, ()> {
        let (left, right) = string.split_once('°').ok_or(())?;
        let mut chars = left.chars();
        let positive = match chars.next() {
            Some('N') => true,
            Some('S') => false,
            _ => return Err(()),
        };
        let degree: u8 = chars.as_str().parse().map_err(|_| ())?;

        let (left, right) = right.split_once('\'').ok_or(())?;
        let minute: u8 = left.parse().map_err(|_| ())?;
        let mut second: u16 = right.parse().map_err(|_| ())?;
        if !(0..=90).contains(&degree) || !(0..60).contains(&minute) || (60..100).contains(&second)
        {
            return Err(());
        }
        if second < 60 {
            second *= 10;
        }
        let value = (degree as i32 * 3600 + minute as i32 * 60) * SUB_SECOND
            + second as i32 * SUB_SECOND / 10;
        Ok(Self(if positive { value } else { -value }))
    }
}

impl<U, const D: char> core::ops::Add<Distance<i32, U>> for Latitude<D>
where
    U: Copy + Into<i32> + Default,
{
    type Output = Self;

    fn add(self, delta: Distance<i32, U>) -> Self {
        let cm = delta.u(CentiMeter).raw as i64;
        let seconds = self.0 + (cm * SUB_SECOND as i64 / 30_92) as i32;
        if seconds.abs() > MAX_SECONDS {
            return Self(-(seconds % MAX_SECONDS));
        }
        Self(seconds)
    }
}

impl<U, const D: char> core::ops::Sub<Distance<i32, U>> for Latitude<D>
where
    U: Copy + Into<i32> + Default,
{
    type Output = Self;

    fn sub(self, delta: Distance<i32, U>) -> Self {
        let cm = delta.u(CentiMeter).raw as i64;
        let seconds = self.0 - (cm * SUB_SECOND as i64 / 30_92) as i32;
        if seconds.abs() > MAX_SECONDS {
            return Self(-(seconds % MAX_SECONDS));
        }
        Self(seconds)
    }
}

impl<const D: char> core::ops::Sub for Latitude<D> {
    type Output = Distance<i32, Meter>;

    fn sub(self, other: Self) -> Distance<i32, Meter> {
        let delta = (self.0 - other.0) as i64;
        Distance::new((delta * 30_92 / 100 / SUB_SECOND as i64) as i32, Meter)
    }
}

impl<const D: char> fmt::Display for Latitude<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ns = if self.0 >= 0 { "N" } else { "S" };
        let sub_second = self.0.abs();
        let second = sub_second / SUB_SECOND;
        let (degree, minute, second) = (second / 3600, (second / 60) % 60, second % 60);
        let sub_second = sub_second % SUB_SECOND;
        write!(f, "{}{:02}{}{:02}'{:03}", ns, degree, D, minute, second * 10 + sub_second / 100)
    }
}

impl<const D: char> serde::Serialize for Latitude<D> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut string = String::<16>::new();
        write!(string, "{}", self).ok();
        serializer.serialize_str(string.as_str())
    }
}

impl<'a> serde::Deserialize<'a> for Latitude<'°'> {
    fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let s = <&str>::deserialize(deserializer)?;
        Latitude::try_from(s).map_err(|_| <D::Error as serde::de::Error>::custom("Not longitude"))
    }
}

mod test {
    #[test]
    fn test_latitude() {
        use core::convert::TryFrom;

        use super::Latitude;
        use crate::types::measurement::{unit::CentiMeter, Distance};

        assert!(Latitude::try_from("N00°00'00").is_ok());
        assert!(Latitude::try_from("N90°00'00").is_ok());
        assert!(Latitude::try_from("S90°00'00").is_ok());
        assert!(Latitude::try_from("N00°00'100").is_ok());
        assert!(Latitude::try_from("N00°00'60").is_err());
        assert!(Latitude::try_from("N00°60'00").is_err());
        assert!(Latitude::try_from("N91°00'00").is_err());
        assert!(Latitude::try_from("S92°00'00").is_err());
        assert!(Latitude::try_from("E00°01'00").is_err());
        assert!(Latitude::try_from("W00°01'00").is_err());

        let latitude = Latitude::try_from("N40°19'48").unwrap();
        assert_eq!("N40°19'480", format!("{}", latitude));
        assert_eq!("N40°19'479", format!("{}", latitude + Distance::new(-7, CentiMeter)));

        let distance = latitude - Latitude::try_from("N40°18'480").unwrap();
        assert_eq!("1855m", format!("{}", distance));
    }
}
