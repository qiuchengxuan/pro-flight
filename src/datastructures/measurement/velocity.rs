use core::ops::{Add, Sub};

#[derive(Copy, Clone, PartialEq)]
pub enum VelocityUnit {
    MilliMeter = 1,
    Meter = 1000,
    Feet = 3300,
    KiloMeter = 1_000_000,
    NauticalMile = 1852 * 1000,
}

impl PartialOrd for VelocityUnit {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (*self as usize).partial_cmp(&(*other as usize))
    }
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Velocity<T: Default + Copy + Clone + PartialEq>(pub T);

impl<T: PartialEq + Copy + Default> PartialEq<T> for Velocity<T> {
    fn eq(&self, rhs: &T) -> bool {
        self.0 == *rhs
    }
}

impl<T: Add<Output = T> + Copy + Default + PartialEq> Add for Velocity<T> {
    type Output = Velocity<T::Output>;
    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl<T: Sub<Output = T> + Copy + Default + PartialEq> Sub for Velocity<T> {
    type Output = Velocity<T::Output>;
    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl<T: sval::value::Value + Copy + Default + PartialEq> sval::value::Value for Velocity<T> {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        self.0.stream(stream)
    }
}

impl<T: Into<isize> + Copy + Default + PartialEq> Velocity<T> {
    #[inline]
    pub fn convert(self, from: VelocityUnit, to: VelocityUnit, scale: isize) -> isize {
        let raw: isize = self.0.into();
        match (from, to) {
            (VelocityUnit::MilliMeter, _) => raw * scale / to as isize,
            (_, VelocityUnit::MilliMeter) => raw * scale * from as isize,
            _ => {
                if from >= to {
                    raw * scale * from as isize / to as isize
                } else {
                    raw * scale * to as isize / from as isize
                }
            }
        }
    }
}

impl<T: Into<isize> + Copy + Default + PartialEq> Into<f32> for Velocity<T> {
    fn into(self) -> f32 {
        let value: isize = self.0.into();
        value as f32 / (VelocityUnit::Meter as isize as f32)
    }
}
