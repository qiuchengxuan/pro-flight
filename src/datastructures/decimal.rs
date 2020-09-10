use core::fmt::{Debug, Display};
use core::num::ParseIntError;
use core::str::FromStr;

#[allow(unused_imports)] // false warning
use micromath::F32Ext;

#[derive(Copy, Clone, Default, PartialEq)]
pub struct IntegerDecimal<I, D> {
    pub integer: I,
    pub decimal: D,
    pub decimal_length: u8,
}

macro_rules! impl_into_f32 {
    () => {};
    (, ($integer_type:tt, $decimal_type:tt) $(, ($integer_types:tt, $decimal_types:tt))*) => {
        impl_into_f32!{ ($integer_type, $decimal_type) $(, ($integer_types, $decimal_types))* }
    };
    (($integer_type:tt, $decimal_type:tt) $(, ($integer_types:tt, $decimal_types:tt))*) => {
        impl Into<f32> for IntegerDecimal<$integer_type, $decimal_type> {
            fn into(self) -> f32 {
                let integer = self.integer as f32;
                let decimal = (self.decimal as f32).copysign(integer);
                integer + decimal * 0.1f32.powf(self.decimal_length as f32)
            }
        }

        impl_into_f32!{ $(, ($integer_types, $decimal_types))* }
    };
}

impl_into_f32! { (u8, u8) }

impl<I: Display, D: Display> Debug for IntegerDecimal<I, D> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}.{}#{}", self.integer, self.decimal, self.decimal_length)
    }
}

macro_rules! impl_from_str {
    () => {};
    (, ($type:tt -> $length:expr) $(, ($types:tt -> $lengths:expr))*) => {
        impl_from_str!{ ($type -> $length) $(, ($types -> $lengths))* }
    };
    (($type:tt -> $length:expr) $(, ($types:tt -> $lengths:expr))*) => {
        impl<I: FromStr<Err = ParseIntError> + Default> From<&str> for IntegerDecimal<I, $type> {
            fn from(string: &str) -> Self {
                if string.len() == 0 {
                    return Self::default();
                }
                let mut splitted = string.split('.');
                let mut integer = I::default();
                if let Some(field) = splitted.next() {
                    integer = field.parse().unwrap_or_default();
                }
                let mut decimal_length = 0;
                let mut decimal = $type::default();
                if let Some(field) = splitted.next() {
                    decimal_length = core::cmp::min(field.len(), $length);
                    decimal = (&field[..decimal_length]).parse().unwrap_or_default();
                }
                Self {
                    integer,
                    decimal,
                    decimal_length: decimal_length as u8,
                }
            }
        }

        impl_from_str!{ $(, ($types -> $lengths))* }
    };
}

impl_from_str! { (u8 -> 2) }

impl<I: Display, D: Into<u32> + Copy + Display> Display for IntegerDecimal<I, D> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if self.decimal_length == 0 {
            return write!(f, "{}", self.integer);
        }
        let mut decimal: u32 = self.decimal.into();
        let mut padding_length: usize = self.decimal_length as usize - 1;
        while decimal / 10 > 0 {
            decimal /= 10;
            padding_length -= 1;
        }
        if padding_length == 0 {
            return write!(f, "{}.{}", self.integer, self.decimal);
        }
        write!(f, "{}.{:padding$}{}", "0", self.integer, self.decimal, padding = padding_length)
    }
}

mod test {
    #[test]
    fn test_display() {
        use super::IntegerDecimal;

        let decimal: IntegerDecimal<u8, u8> = IntegerDecimal::from("0");
        assert_eq!("0", format!("{}", decimal));
        let decimal: IntegerDecimal<u8, u8> = IntegerDecimal::from("0.0");
        assert_eq!("0.0", format!("{}", decimal));
        let decimal: IntegerDecimal<u8, u8> = IntegerDecimal::from("0.1");
        assert_eq!("0.1", format!("{}", decimal));
        let decimal: IntegerDecimal<u8, u8> = IntegerDecimal::from("0.01");
        assert_eq!("0.01", format!("{}", decimal));
        let decimal: IntegerDecimal<u8, u8> = IntegerDecimal::from("0.11");
        assert_eq!("0.11", format!("{}", decimal));
    }
}
