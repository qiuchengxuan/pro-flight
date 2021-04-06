// TODO: Replace with public fixed-point crate
use core::fmt::Display;
use core::str::FromStr;

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

impl<T: Copy + From<i32> + Into<i32>, const D: usize> FixedPoint<T, D> {
    pub fn integer(self) -> T {
        let value = self.0.into();
        T::from(value / self.exp() as i32)
    }

    pub fn decimal(self) -> T {
        let value = self.0.into();
        T::from(value % self.exp() as i32)
    }
}

impl<T: Copy + Into<i32>, const D: usize> Into<f32> for FixedPoint<T, D> {
    fn into(self) -> f32 {
        let value: i32 = self.0.into();
        value as f32 / self.exp() as f32
    }
}

impl<T: From<i32>, const D: usize> FromStr for FixedPoint<T, D> {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, ()> {
        let mut splitted = string.split('.');
        let mut integer = splitted.next().ok_or(())?.parse::<i32>().map_err(|_| ())?;
        integer *= 10_i32.pow(D as u32);
        let field = match splitted.next() {
            Some(s) => s,
            None => return Ok(Self(T::from(integer))),
        };
        let decimal_length = core::cmp::min(field.len(), 255);
        let mut decimal = field.parse::<i32>().map_err(|_| ())?;
        if integer < 0 {
            decimal = -decimal
        }
        if D >= decimal_length {
            decimal *= 10_i32.pow((D - decimal_length) as u32);
        } else {
            decimal /= 10_i32.pow((decimal_length - D) as u32);
        }
        Ok(Self(T::from(integer + decimal)))
    }
}

impl<T: Copy + From<i32> + Into<i32> + Display, const D: usize> Display for FixedPoint<T, D> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if D == 0 {
            return write!(f, "{}", self.integer());
        }
        write!(f, "{}.{:0length$}", self.integer(), self.decimal().into().abs(), length = D)
    }
}

mod test {
    #[test]
    fn test_fixed_point() {
        use super::FixedPoint;

        let decimal: FixedPoint<i32, 0> = "0".parse().unwrap();
        assert_eq!("0", format!("{}", decimal));
        let decimal: FixedPoint<i32, 1> = "0.0".parse().unwrap();
        assert_eq!("0.0", format!("{}", decimal));
        let decimal: FixedPoint<i32, 1> = "0.1".parse().unwrap();
        assert_eq!("0.1", format!("{}", decimal));
        let decimal: FixedPoint<i32, 2> = "0.01".parse().unwrap();
        assert_eq!("0.01", format!("{}", decimal));
        let decimal: FixedPoint<i32, 2> = "0.11".parse().unwrap();
        assert_eq!("0.11", format!("{}", decimal));
        let decimal: FixedPoint<i32, 2> = "0.1".parse().unwrap();
        assert_eq!("0.10", format!("{}", decimal));
        let decimal: FixedPoint<i32, 2> = "1".parse().unwrap();
        assert_eq!("1.00", format!("{}", decimal));
        let decimal: FixedPoint<i32, 2> = "1.001".parse().unwrap();
        assert_eq!("1.00", format!("{}", decimal));
        let decimal: FixedPoint<i32, 3> = "0.001".parse().unwrap();
        assert_eq!("0.001", format!("{}", decimal));
    }
}
