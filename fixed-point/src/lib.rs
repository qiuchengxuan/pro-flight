#![cfg_attr(not(test), no_std)]

pub use fixed_point_macro::fixed_point;

use core::fmt::Display;
use core::ops::{Add, Div, Mul, Rem};
use core::str::FromStr;
use num_traits::pow::Pow;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct FixedPoint<T, const D: usize>(pub T);

impl<T, const D: usize> FixedPoint<T, D> {
    pub fn decimal_length(self) -> usize {
        D
    }

    pub fn exp(self) -> usize {
        10_usize.pow(D as u32)
    }
}

impl<T, const D: usize> FixedPoint<T, D>
where
    T: From<u8> + Pow<u32, Output = T> + Mul<Output = T> + Add<Output = T>,
{
    pub fn new(integer: T, decimal: T) -> Self {
        Self(integer * T::from(10).pow(D as u32) + decimal)
    }
}

pub trait Number:
    Copy + From<u8> + Div<Output = Self> + Pow<u32, Output = Self> + Rem<Output = Self>
{
}

impl<T> Number for T where
    T: Copy + From<u8> + Div<Output = T> + Pow<u32, Output = T> + Rem<Output = T>
{
}

impl<T: Number, const D: usize> FixedPoint<T, D> {
    pub fn integer(&self) -> T {
        self.0 / (T::from(10)).pow(D as u32)
    }

    pub fn decimal(&self) -> T {
        self.0 % (T::from(10)).pow(D as u32)
    }
}

impl<T: Copy + Into<i32>, const D: usize> Into<f32> for FixedPoint<T, D> {
    fn into(self) -> f32 {
        let value: i32 = self.0.into();
        value as f32 / self.exp() as f32
    }
}

macro_rules! impl_unsigned_from_str {
    ($type:ty) => {
        impl<const D: usize> FromStr for FixedPoint<$type, D> {
            type Err = <$type as FromStr>::Err;
            fn from_str(string: &str) -> Result<Self, Self::Err> {
                let mut splitted = string.split('.');
                let mut integer = splitted.next().unwrap_or("").parse::<$type>()?;
                integer *= (10 as $type).pow(D as u32);
                let field = match splitted.next() {
                    Some(s) => s,
                    None => return Ok(Self(integer)),
                };
                let decimal_length = core::cmp::min(field.len(), 255);
                let mut decimal = field.parse::<$type>()?;
                if D >= decimal_length {
                    decimal *= (10 as $type).pow((D - decimal_length) as u32);
                } else {
                    decimal /= (10 as $type).pow((decimal_length - D) as u32);
                }
                Ok(Self(integer + decimal))
            }
        }
    };
}

impl_unsigned_from_str!(u8);
impl_unsigned_from_str!(u16);

macro_rules! impl_signed_from_str {
    ($type:ty) => {
        impl<const D: usize> FromStr for FixedPoint<$type, D> {
            type Err = <$type as FromStr>::Err;
            fn from_str(string: &str) -> Result<Self, Self::Err> {
                let mut splitted = string.split('.');
                let mut integer = splitted.next().unwrap_or("").parse::<$type>()?;
                integer *= (10 as $type).pow(D as u32);
                let field = match splitted.next() {
                    Some(s) => s,
                    None => return Ok(Self(integer)),
                };
                let decimal_length = core::cmp::min(field.len(), 255);
                let mut decimal = field.parse::<$type>()?;
                if integer < 0 {
                    decimal = -decimal
                }
                if D >= decimal_length {
                    decimal *= (10 as $type).pow((D - decimal_length) as u32);
                } else {
                    decimal /= (10 as $type).pow((decimal_length - D) as u32);
                }
                Ok(Self(integer + decimal))
            }
        }
    };
}

impl_signed_from_str!(i32);

impl<T: Number + Display + Into<i32>, const D: usize> Display for FixedPoint<T, D> {
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
        write!(f, "{}.{:0length$}", self.integer(), decimal, length = length)
    }
}

impl<T: Copy + Into<i32>, const D: usize> serde::Serialize for FixedPoint<T, D> {
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
    }
}