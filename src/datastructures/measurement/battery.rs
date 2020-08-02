#[derive(Copy, Clone, Default, Debug)]
pub struct Battery(pub u16); // unit of milli voltage

impl sval::value::Value for Battery {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.any(self.0)
    }
}

impl core::ops::Div<u16> for Battery {
    type Output = Self;
    fn div(self, div: u16) -> Self {
        Self(self.0 / div)
    }
}

impl From<u16> for Battery {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl Battery {
    pub fn percentage(&self) -> u8 {
        let result = match self.0 {
            0..=3290 => 0,
            3300..=3499 => (self.0 as usize - 3300) * 5 / (3499 - 3300),
            3500..=3679 => (self.0 as usize - 3500) * 5 / (3679 - 3500) + 5,
            3680..=3699 => (self.0 as usize - 3680) * 5 / (3699 - 3680) + 10,
            3700..=3729 => (self.0 as usize - 3700) * 5 / (3729 - 3700) + 15,
            3730..=3769 => (self.0 as usize - 3730) * 10 / (3769 - 3730) + 20,
            3770..=3789 => (self.0 as usize - 3770) * 10 / (3499 - 3300) + 30,
            3790..=3819 => (self.0 as usize - 3790) * 10 / (3819 - 3790) + 40,
            3820..=3869 => (self.0 as usize - 3820) * 10 / (3869 - 3820) + 50,
            3870..=3929 => (self.0 as usize - 3870) * 10 / (3929 - 3870) + 60,
            3930..=3999 => (self.0 as usize - 3930) * 10 / (3999 - 3930) + 70,
            4000..=4079 => (self.0 as usize - 4000) * 10 / (4079 - 4000) + 80,
            4080..=4199 => (self.0 as usize - 4080) * 10 / (4199 - 4080) + 90,
            _ => 100,
        };
        result as u8
    }
}

mod test {
    #[test]
    fn test_battery_percentage() {
        use super::Battery;

        assert_eq!(Battery(4200).percentage(), 100);
        assert_eq!(Battery(4100).percentage(), 91);
        assert_eq!(Battery(4000).percentage(), 80);
        assert_eq!(Battery(3900).percentage(), 65);
        assert_eq!(Battery(3800).percentage(), 43);
        assert_eq!(Battery(3700).percentage(), 15);
        assert_eq!(Battery(3600).percentage(), 7);
        assert_eq!(Battery(3500).percentage(), 5);
        assert_eq!(Battery(3400).percentage(), 2);
        assert_eq!(Battery(3300).percentage(), 0);
    }
}
