use core::{cmp, str::FromStr};

use fixed_point::FixedPoint;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Voltage(pub FixedPoint<u16, 3>); // voltage

#[macro_export]
macro_rules! voltage {
    ($v:expr) => {
        Voltage(fixed_point::fixed!($v, 3))
    };
}

impl serde::Serialize for Voltage {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl core::ops::Div<u16> for Voltage {
    type Output = Self;

    fn div(self, div: u16) -> Self {
        Self(self.0 / div)
    }
}

impl FromStr for Voltage {
    type Err = <FixedPoint<u16, 3> as FromStr>::Err;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        FixedPoint::from_str(string).map(|v| Self(v))
    }
}

impl From<u16> for Voltage {
    fn from(value: u16) -> Self {
        Self(FixedPoint(value))
    }
}

impl Into<u16> for Voltage {
    fn into(self) -> u16 {
        self.0.0
    }
}

impl Voltage {
    pub fn cells(&self) -> u8 {
        cmp::max(1, cmp::min(self.0.0 / (4200 + (3300 * 2 - 4200) / 2) + 1, 8)) as u8
    }

    pub fn soc(&self) -> u8 {
        let result = match self.0.0 / self.cells() as u16 {
            0..=3290 => 0,
            3300..=3499 => (self.0.0 as usize - 3300) * 5 / (3499 - 3300),
            3500..=3679 => (self.0.0 as usize - 3500) * 5 / (3679 - 3500) + 5,
            3680..=3699 => (self.0.0 as usize - 3680) * 5 / (3699 - 3680) + 10,
            3700..=3729 => (self.0.0 as usize - 3700) * 5 / (3729 - 3700) + 15,
            3730..=3769 => (self.0.0 as usize - 3730) * 10 / (3769 - 3730) + 20,
            3770..=3789 => (self.0.0 as usize - 3770) * 10 / (3499 - 3300) + 30,
            3790..=3819 => (self.0.0 as usize - 3790) * 10 / (3819 - 3790) + 40,
            3820..=3869 => (self.0.0 as usize - 3820) * 10 / (3869 - 3820) + 50,
            3870..=3929 => (self.0.0 as usize - 3870) * 10 / (3929 - 3870) + 60,
            3930..=3999 => (self.0.0 as usize - 3930) * 10 / (3999 - 3930) + 70,
            4000..=4079 => (self.0.0 as usize - 4000) * 10 / (4079 - 4000) + 80,
            4080..=4199 => (self.0.0 as usize - 4080) * 10 / (4199 - 4080) + 90,
            _ => 100,
        };
        result as u8
    }
}

mod test {
    #[test]
    fn test_voltage_soc() {
        use fixed_point::FixedPoint;

        use super::Voltage;

        assert_eq!(Voltage(FixedPoint(4200)).soc(), 100);
        assert_eq!(Voltage(FixedPoint(4100)).soc(), 91);
        assert_eq!(Voltage(FixedPoint(4000)).soc(), 80);
        assert_eq!(Voltage(FixedPoint(3900)).soc(), 65);
        assert_eq!(Voltage(FixedPoint(3800)).soc(), 43);
        assert_eq!(Voltage(FixedPoint(3700)).soc(), 15);
        assert_eq!(Voltage(FixedPoint(3600)).soc(), 7);
        assert_eq!(Voltage(FixedPoint(3500)).soc(), 5);
        assert_eq!(Voltage(FixedPoint(3400)).soc(), 2);
        assert_eq!(Voltage(FixedPoint(3300)).soc(), 0);
    }
}
