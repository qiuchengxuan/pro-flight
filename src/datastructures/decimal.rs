use core::fmt::Display;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct IntegerDecimal(pub isize);

impl IntegerDecimal {
    pub fn new(value: isize, decimal_length: u8) -> Self {
        Self(value << 8 | decimal_length as isize)
    }

    pub fn decimal_length(self) -> u8 {
        self.0 as u8
    }

    pub fn exp(self) -> usize {
        let decimal_length = self.0 as u8;
        10_usize.pow(decimal_length as u32)
    }

    pub fn integer(self) -> isize {
        let number = self.0 >> 8;
        number / self.exp() as isize
    }

    pub fn decimal(self) -> isize {
        let number = self.0 >> 8;
        number % self.exp() as isize
    }
}

impl Into<f32> for IntegerDecimal {
    fn into(self) -> f32 {
        let number = self.0 >> 8;
        number as f32 / self.exp() as f32
    }
}

impl From<&str> for IntegerDecimal {
    fn from(string: &str) -> Self {
        if string == "" {
            return Self::default();
        }
        let mut splitted = string.split('.');
        let mut integer = 0;
        if let Some(field) = splitted.next() {
            integer = field.parse().unwrap_or_default();
        }
        let mut decimal_length = 0;
        let mut decimal = 0;
        if let Some(field) = splitted.next() {
            decimal_length = core::cmp::min(field.len(), 255);
            decimal = field.parse().unwrap_or_default();
            if integer < 0 {
                decimal = -decimal
            }
        }
        let exp = 10_isize.pow(decimal_length as u32);
        Self::new(integer * exp + decimal, decimal_length as u8)
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

        let decimal = IntegerDecimal::from("0");
        assert_eq!("0", format!("{}", decimal));
        let decimal = IntegerDecimal::from("0.0");
        assert_eq!("0.0", format!("{}", decimal));
        let decimal = IntegerDecimal::from("0.1");
        assert_eq!("0.1", format!("{}", decimal));
        let decimal = IntegerDecimal::from("0.01");
        assert_eq!("0.01", format!("{}", decimal));
        let decimal = IntegerDecimal::from("0.11");
        assert_eq!("0.11", format!("{}", decimal));

        let decimal = IntegerDecimal::from("0.001");
        assert_eq!("0.001", format!("{}", decimal));
    }
}
