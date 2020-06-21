use nalgebra::Vector3;

use crate::datastructures::measurement::Altitude;

#[derive(Copy, Clone, Debug, PartialEq, Default, Value)]
pub struct Axis {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Axis {
    pub fn average(self: &Self, other: &Self) -> Self {
        Self { x: (self.x + other.x) / 2, y: (self.y + other.y) / 2, z: (self.z + other.z) / 2 }
    }

    pub fn calibrated(&mut self, calibration: &Self) {
        self.x -= calibration.x;
        self.y -= calibration.y;
        self.z -= calibration.z;
    }
}

impl PartialOrd for Axis {
    fn partial_cmp(self: &Self, other: &Self) -> Option<core::cmp::Ordering> {
        if self.x > other.x || self.y > other.y || self.z > other.z {
            Some(core::cmp::Ordering::Greater)
        } else {
            Some(core::cmp::Ordering::Less)
        }
    }
}

#[derive(Copy, Clone, PartialEq, Value)]
pub struct Measurement {
    pub axes: Axis,
    pub sensitive: i32,
}

impl PartialOrd for Measurement {
    fn partial_cmp(self: &Self, other: &Self) -> Option<core::cmp::Ordering> {
        self.axes.partial_cmp(&other.axes)
    }
}

impl Into<(f32, f32, f32)> for Measurement {
    fn into(self) -> (f32, f32, f32) {
        (
            (self.axes.x as f32 / self.sensitive as f32),
            (self.axes.y as f32 / self.sensitive as f32),
            (self.axes.z as f32 / self.sensitive as f32),
        )
    }
}

impl Into<Vector3<f32>> for Measurement {
    fn into(self) -> Vector3<f32> {
        Vector3::new(
            self.axes.x as f32 / self.sensitive as f32,
            self.axes.y as f32 / self.sensitive as f32,
            self.axes.z as f32 / self.sensitive as f32,
        )
    }
}

impl Default for Measurement {
    fn default() -> Self {
        Self { axes: Default::default(), sensitive: 1 }
    }
}

pub type Acceleration = Measurement;
pub type Gyro = Measurement;
pub type Temperature = i16;

#[derive(Copy, Clone, Default)]
pub struct Pressure(pub u32); // unit of Pa

impl Pressure {
    pub fn to_sea_level_altitude(self) -> Altitude {
        Altitude((1013 - (self.0 / 100) as i32) * 82)
    }
}

#[derive(Copy, Clone, Default)]
pub struct Battery(pub u16); // unit of milli voltage

impl sval::value::Value for Battery {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.any(self.0 as u64)
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
