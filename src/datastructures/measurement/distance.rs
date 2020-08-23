use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::{Add, Mul, Sub};

#[derive(Copy, Clone, Default, Debug)]
pub struct Distance<T: Default + Copy + Clone, U> {
    value: T,
    unit: PhantomData<U>,
}

impl<T: Copy + Default, U> Distance<T, U> {
    pub fn new(value: T, _: U) -> Self {
        Self { value, unit: PhantomData }
    }

    pub fn value(self) -> T {
        self.value
    }

    #[inline]
    pub fn convert<V: Copy + Default>(self, convert: impl Fn(T) -> V) -> Distance<V, U> {
        Distance { value: convert(self.value), unit: PhantomData }
    }
}

impl<T: From<u8> + Copy + Default + PartialEq, U> Distance<T, U> {
    pub fn is_zero(&self) -> bool {
        self.value == T::from(0)
    }
}

impl<T: PartialEq + Copy + Default, U> PartialEq for Distance<T, U> {
    fn eq(&self, rhs: &Self) -> bool {
        self.value == rhs.value
    }
}

impl<T: Add<Output = T> + Copy + Default + PartialEq, U> Add for Distance<T, U> {
    type Output = Distance<T::Output, U>;
    fn add(self, other: Self) -> Self::Output {
        Self { value: self.value + other.value, unit: PhantomData }
    }
}

impl<T: Sub<Output = T> + Copy + Default + PartialEq, U> Sub for Distance<T, U> {
    type Output = Distance<T::Output, U>;
    fn sub(self, other: Self) -> Self::Output {
        Self { value: self.value - other.value, unit: PhantomData }
    }
}

impl<T: Mul<Output = T> + Copy + Default + PartialEq, U> Mul<T> for Distance<T, U> {
    type Output = Distance<T::Output, U>;
    fn mul(self, t: T) -> Self::Output {
        Self { value: self.value * t, unit: PhantomData }
    }
}

impl<T: sval::value::Value + Copy + Default + PartialEq, U> sval::value::Value for Distance<T, U> {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        self.value.stream(stream)
    }
}

impl<F: Default + Into<i32>> Distance<i32, F> {
    pub fn to_unit<T: Default + Into<i32>>(self, _: T) -> Distance<i32, T> {
        let from: i32 = F::default().into();
        let to: i32 = T::default().into();
        Distance { value: self.value * from / to, unit: PhantomData }
    }
}

impl<F: Default + Into<i32>> Distance<u32, F> {
    pub fn to_unit<T: Default + Into<i32>>(self, _: T) -> Distance<u32, T> {
        let from: i32 = F::default().into();
        let to: i32 = T::default().into();
        Distance { value: self.value * from as u32 / to as u32, unit: PhantomData }
    }
}

impl<F: Default + Into<i32>> Distance<f32, F> {
    pub fn to_unit<T: Default + Into<i32>>(self, _: T) -> Distance<f32, T> {
        let ratio = F::default().into() as f32 / T::default().into() as f32;
        Distance { value: self.value * ratio, unit: PhantomData }
    }
}
