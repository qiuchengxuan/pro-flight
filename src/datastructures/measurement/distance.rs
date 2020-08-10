use core::ops::{Add, Sub};

#[derive(Copy, Clone, PartialEq)]
pub enum DistanceUnit {
    CentiMeter = 1,
    Feet = 330,
    Meter = 100,
    KiloMeter = 100_000,
    NauticalMile = 1852 * 100,
}

impl PartialOrd for DistanceUnit {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (*self as usize).partial_cmp(&(*other as usize))
    }
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Distance<T: Default + Copy + Clone + PartialEq>(pub T);

impl<T: PartialEq + Copy + Default> PartialEq<T> for Distance<T> {
    fn eq(&self, rhs: &T) -> bool {
        self.0 == *rhs
    }
}

impl<T: Add<Output = T> + Copy + Default + PartialEq> Add for Distance<T> {
    type Output = Distance<T::Output>;
    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl<T: Sub<Output = T> + Copy + Default + PartialEq> Sub for Distance<T> {
    type Output = Distance<T::Output>;
    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl<T: sval::value::Value + Copy + Default + PartialEq> sval::value::Value for Distance<T> {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        self.0.stream(stream)
    }
}

impl<T: Into<i32> + Copy + Default + PartialEq> Distance<T> {
    #[inline]
    pub fn convert(self, from: DistanceUnit, to: DistanceUnit, scale: i32) -> i32 {
        let raw: i32 = self.0.into();
        match (from, to) {
            (DistanceUnit::CentiMeter, _) => raw * scale / to as i32,
            (_, DistanceUnit::CentiMeter) => raw * scale * from as i32,
            _ => {
                if from >= to {
                    raw * scale * from as i32 / to as i32
                } else {
                    raw * scale * to as i32 / from as i32
                }
            }
        }
    }
}

impl<T: Into<i32> + Copy + Default + PartialEq> Into<f32> for Distance<T> {
    fn into(self) -> f32 {
        let value: i32 = self.0.into();
        value as f32 / (DistanceUnit::Meter as i32 as f32)
    }
}
