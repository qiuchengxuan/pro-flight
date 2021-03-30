use core::fmt::{Debug, Display};
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Div, Mul, Sub};

#[derive(Copy, Clone, Default, Debug)]
pub struct Distance<T, U> {
    pub value: T,
    unit: PhantomData<U>,
}

impl<T: Display, U: Display + Default> core::fmt::Display for Distance<T, U> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}{}", self.value, U::default())
    }
}

impl<T: Copy + Default, U: Copy> Distance<T, U> {
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

impl<T: From<u8> + Copy + Default + PartialEq, U: Copy> Distance<T, U> {
    pub fn is_zero(&self) -> bool {
        self.value == T::from(0)
    }
}

impl<T: PartialEq + Copy + Default, U: Copy> PartialEq for Distance<T, U> {
    fn eq(&self, rhs: &Self) -> bool {
        self.value == rhs.value
    }
}

impl<T: Add<Output = T> + Copy + Default + PartialEq, U: Copy> Add for Distance<T, U> {
    type Output = Distance<T::Output, U>;
    fn add(self, other: Self) -> Self::Output {
        Self { value: self.value + other.value, unit: PhantomData }
    }
}

impl<T: AddAssign + Copy + Default + PartialEq, U: Copy> AddAssign for Distance<T, U> {
    fn add_assign(&mut self, other: Self) {
        self.value += other.value
    }
}

impl<T: Sub<Output = T> + Copy + Default + PartialEq, U: Copy> Sub for Distance<T, U> {
    type Output = Distance<T::Output, U>;
    fn sub(self, other: Self) -> Self::Output {
        Self { value: self.value - other.value, unit: PhantomData }
    }
}

impl<T: Mul<Output = T> + Copy + Default + PartialEq, U: Copy> Mul<T> for Distance<T, U> {
    type Output = Distance<T::Output, U>;
    fn mul(self, t: T) -> Self::Output {
        Self { value: self.value * t, unit: PhantomData }
    }
}

impl<T: Div<Output = T> + Copy + Default + PartialEq, U: Copy> Div<T> for Distance<T, U> {
    type Output = Distance<T::Output, U>;
    fn div(self, t: T) -> Self::Output {
        Self { value: self.value / t, unit: PhantomData }
    }
}

impl<T: sval::Value + Copy + Default + PartialEq, U: Copy> sval::Value for Distance<T, U> {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        self.value.stream(stream)
    }
}

impl<V, F: Copy + Default + Into<V>> Distance<V, F>
where
    V: Mul<Output = V> + Div<Output = V> + Copy + Default,
{
    pub fn to_unit<T: Copy + Default + Into<V>>(self, _: T) -> Distance<V, T> {
        let from: V = F::default().into();
        let to: V = T::default().into();
        Distance { value: self.value * from / to, unit: PhantomData }
    }
}
