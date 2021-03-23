use core::fmt::Display;
use core::str::FromStr;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct IntegerDecimal(pub i32);

macro_rules! integer_decimal {
    ($integer:expr, $decimal_length:expr) => {
        IntegerDecimal($integer << 8 | $decimal_length as i32)
    };
}

impl IntegerDecimal {
    pub fn new(value: i32, decimal_length: u8) -> Self {
        Self(value << 8 | decimal_length as i32)
    }

    pub fn decimal_length(self) -> u8 {
        self.0 as u8
    }

    pub fn exp(self) -> u32 {
        let decimal_length = self.0 as u8;
        10_u32.pow(decimal_length as u32)
    }

    pub fn integer(self) -> i32 {
        let number = self.0 >> 8;
        number / self.exp() as i32
    }

    pub fn decimal(self) -> i32 {
        let number = self.0 >> 8;
        number % self.exp() as i32
    }
}

impl Into<f32> for IntegerDecimal {
    fn into(self) -> f32 {
        let number = self.0 >> 8;
        number as f32 / self.exp() as f32
    }
}

impl FromStr for IntegerDecimal {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, ()> {
        let mut splitted = string.split('.');
        let integer = splitted.next().ok_or(())?.parse::<i32>().map_err(|_| ())?;
        let field = match splitted.next() {
            Some(s) => s,
            None => return Ok(Self::new(integer, 0)),
        };
        let decimal_length = core::cmp::min(field.len(), 255);
        let mut decimal = field.parse::<i32>().map_err(|_| ())?;
        if integer < 0 {
            decimal = -decimal
        }
        let exp = 10_i32.pow(decimal_length as u32);
        Ok(Self::new(integer * exp + decimal, decimal_length as u8))
    }
}

impl Display for IntegerDecimal {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let decimal_length = self.decimal_length() as usize;
        if decimal_length == 0 {
            return write!(f, "{}", self.integer());
        }
        write!(f, "{}.{:0length$}", self.integer(), self.decimal().abs(), length = decimal_length)
    }
}

mod test {
    #[test]
    fn test_display() {
        use super::IntegerDecimal;

        let mut decimal: IntegerDecimal = "0".parse().unwrap();
        assert_eq!("0", format!("{}", decimal));
        decimal = "0.0".parse().unwrap();
        assert_eq!("0.0", format!("{}", decimal));
        decimal = "0.1".parse().unwrap();
        assert_eq!("0.1", format!("{}", decimal));
        decimal = "0.01".parse().unwrap();
        assert_eq!("0.01", format!("{}", decimal));
        decimal = "0.11".parse().unwrap();
        assert_eq!("0.11", format!("{}", decimal));

        decimal = "0.001".parse().unwrap();
        assert_eq!("0.001", format!("{}", decimal));
    }
}
