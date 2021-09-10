#![cfg_attr(not(test), no_std)]

pub use fixed_point_macro::fixed_point;

use core::convert;
use core::fmt::Display;
use core::ops;
use core::str::FromStr;
use num_traits::pow::Pow;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct FixedPoint<T, const D: u8>(pub T);

impl<T, const D: u8> FixedPoint<T, D> {
    pub fn decimal_length(self) -> u8 {
        D
    }

    pub fn exp(self) -> usize {
        10_usize.pow(D as u32)
    }
}

impl<T, const D: u8> FixedPoint<T, D>
where
    T: From<u8> + Pow<u32, Output = T> + ops::Mul<Output = T> + ops::Add<Output = T>,
{
    pub fn new(integer: T, decimal: T) -> Self {
        Self(integer * T::from(10).pow(D as u32) + decimal)
    }
}

pub trait Number:
    Copy + From<u8> + ops::Div<Output = Self> + Pow<u32, Output = Self> + ops::Rem<Output = Self>
{
}

impl<T> Number for T where
    T: Copy + From<u8> + ops::Div<Output = T> + Pow<u32, Output = T> + ops::Rem<Output = T>
{
}

impl<T: Number, const D: u8> FixedPoint<T, D> {
    pub fn integer(&self) -> T {
        self.0 / (T::from(10)).pow(D as u32)
    }

    pub fn decimal(&self) -> T {
        self.0 % (T::from(10)).pow(D as u32)
    }
}

impl<T: ops::Div<Output = T>, const D: u8> ops::Div<T> for FixedPoint<T, D> {
    type Output = Self;
    fn div(self, div: T) -> Self {
        Self(self.0 / div)
    }
}

impl<T: Copy + Into<i32>, const D: u8> Into<f32> for FixedPoint<T, D> {
    fn into(self) -> f32 {
        let value: i32 = self.0.into();
        value as f32 / self.exp() as f32
    }
}

impl<T: convert::TryFrom<isize>, const D: u8> FromStr for FixedPoint<T, D> {
    type Err = ();
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let negative = string.chars().next().map(|c| c == '-').unwrap_or(false);
        let mut splitted = string.split('.');
        let mut integer = splitted.next().ok_or(())?.parse::<isize>().map_err(|_| ())?;
        integer *= (10 as isize).pow(D as u32);
        let field = match splitted.next() {
            Some(s) => s,
            None => return T::try_from(integer).map(|v| Self(v)).map_err(|_| ()),
        };
        let decimal_length = core::cmp::min(field.len(), 255) as u8;
        let mut decimal = field.parse::<isize>().map_err(|_| ())?;
        if integer < 0 || negative {
            decimal = -decimal
        }
        if D >= decimal_length {
            decimal *= (10 as isize).pow((D - decimal_length) as u32);
        } else {
            decimal /= (10 as isize).pow((decimal_length - D) as u32);
        }
        T::try_from(integer + decimal).map(|v| Self(v)).map_err(|_| ())
    }
}

impl<T, const D: u8> Display for FixedPoint<T, D>
where
    T: Number + Display + Into<i32> + PartialEq + From<u8> + PartialOrd,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut decimal = self.decimal().into().abs();
        if D == 0 || decimal == 0 {
            return write!(f, "{}.0", self.integer());
        }
        let mut length = D;
        while decimal % 10 == 0 {
            decimal = decimal / 10;
            length -= 1;
        }
        let integer = self.integer();
        if integer == T::from(0) && self.0 < T::from(0) {
            write!(f, "-0.{:0length$}", decimal, length = length as usize)
        } else {
            write!(f, "{}.{:0length$}", integer, decimal, length = length as usize)
        }
    }
}

impl<T: Copy + Into<i32>, const D: u8> serde::Serialize for FixedPoint<T, D> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f32((*self).into())
    }
}

mod test {
    #[test]
    fn test_fixed_point() {
        use super::FixedPoint;

        let decimal: FixedPoint<i32, 0> = "0".parse().unwrap();
        assert_eq!("0.0", format!("{}", decimal));
        let decimal: FixedPoint<i32, 1> = "0.0".parse().unwrap();
        assert_eq!("0.0", format!("{}", decimal));
        let decimal: FixedPoint<i32, 1> = "0.1".parse().unwrap();
        assert_eq!("0.1", format!("{}", decimal));
        let decimal: FixedPoint<i32, 2> = "0.01".parse().unwrap();
        assert_eq!("0.01", format!("{}", decimal));
        let decimal: FixedPoint<i32, 2> = "0.11".parse().unwrap();
        assert_eq!("0.11", format!("{}", decimal));
        let decimal: FixedPoint<i32, 2> = "0.1".parse().unwrap();
        assert_eq!("0.1", format!("{}", decimal));
        let decimal: FixedPoint<i32, 2> = "1".parse().unwrap();
        assert_eq!("1.0", format!("{}", decimal));
        let decimal: FixedPoint<i32, 2> = "1.001".parse().unwrap();
        assert_eq!("1.0", format!("{}", decimal));
        let decimal: FixedPoint<i32, 3> = "0.001".parse().unwrap();
        assert_eq!("0.001", format!("{}", decimal));
        let decimal: FixedPoint<i32, 3> = "0.0001".parse().unwrap();
        assert_eq!("0.0", format!("{}", decimal));
        let decimal: FixedPoint<i32, 3> = "-0.1".parse().unwrap();
        assert_eq!("-0.1", format!("{}", decimal));
        let decimal: FixedPoint<i32, 3> = "-1.1".parse().unwrap();
        assert_eq!("-1.1", format!("{}", decimal));
    }

    #[test]
    fn test_fixed_point_macro() {
        use fixed_point_macro::fixed_point;

        use super::FixedPoint;

        let decimal = fixed_point!(0.0, 2u16);
        assert_eq!("0.0", format!("{}", decimal));
        let decimal = fixed_point!(0.1, 2u16);
        assert_eq!("0.1", format!("{}", decimal));
        let decimal = fixed_point!(0.11, 2u16);
        assert_eq!("0.11", format!("{}", decimal));
        let decimal = fixed_point!(1.0, 2u16);
        assert_eq!("1.0", format!("{}", decimal));
        let decimal = fixed_point!(1.01, 2u16);
        assert_eq!("1.01", format!("{}", decimal));
        let decimal = fixed_point!(1.10, 2u16);
        assert_eq!("1.1", format!("{}", decimal));
        let decimal = fixed_point!(-0.1, 2i16);
        assert_eq!("-0.1", format!("{}", decimal));
        let decimal = fixed_point!(-1.1, 2i16);
        assert_eq!("-1.1", format!("{}", decimal));
    }

    #[test]
    fn test_malformed() {
        use super::FixedPoint;

        assert_eq!(Err(()), "".parse::<FixedPoint<u16, 4>>());
        assert_eq!(Err(()), "1.".parse::<FixedPoint<u16, 4>>());
        assert_eq!(Err(()), ".1".parse::<FixedPoint<u16, 4>>());
        assert_eq!(Err(()), "-1.0".parse::<FixedPoint<u16, 4>>());
        assert_eq!(Err(()), "10.0".parse::<FixedPoint<u16, 4>>());
    }
}
