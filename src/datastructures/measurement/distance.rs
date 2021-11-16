use core::{
    fmt, marker,
    ops::{Add, AddAssign, Div, Mul, Sub},
};

#[derive(Copy, Clone, Default, Debug)]
pub struct Distance<T, U> {
    pub value: T,
    unit: marker::PhantomData<U>,
}

impl<T: Copy + Default, U: Copy> Distance<T, U> {
    pub fn new(value: T, _: U) -> Self {
        Self { value, unit: core::marker::PhantomData }
    }

    pub fn value(self) -> T {
        self.value
    }

    #[inline]
    pub fn convert<V: Copy + Default>(self, convert: impl Fn(T) -> V) -> Distance<V, U> {
        Distance { value: convert(self.value), unit: marker::PhantomData }
    }
}

impl<T: fmt::Display, U: fmt::Display + Default> fmt::Display for Distance<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.value, U::default())
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
        Self { value: self.value + other.value, unit: marker::PhantomData }
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
        Self { value: self.value - other.value, unit: marker::PhantomData }
    }
}

impl<T: Mul<Output = T> + Copy + Default + PartialEq, U: Copy> Mul<T> for Distance<T, U> {
    type Output = Distance<T::Output, U>;

    fn mul(self, t: T) -> Self::Output {
        Self { value: self.value * t, unit: marker::PhantomData }
    }
}

impl<T: Div<Output = T> + Copy + Default + PartialEq, U: Copy> Div<T> for Distance<T, U> {
    type Output = Distance<T::Output, U>;

    fn div(self, t: T) -> Self::Output {
        Self { value: self.value / t, unit: marker::PhantomData }
    }
}

impl<V, F: Copy + Default + Into<V>> Distance<V, F>
where
    V: Mul<Output = V> + Div<Output = V> + Copy + Default,
{
    pub fn to_unit<T: Copy + Default + Into<V>>(self, _: T) -> Distance<V, T> {
        let from: V = F::default().into();
        let to: V = T::default().into();
        Distance { value: self.value * from / to, unit: marker::PhantomData }
    }
}

impl<T: serde::Serialize, U: Copy> serde::Serialize for Distance<T, U> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(serializer)
    }
}

impl<'a, T: serde::Deserialize<'a>, U> serde::Deserialize<'a> for Distance<T, U> {
    fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self { value: T::deserialize(deserializer)?, unit: marker::PhantomData })
    }
}
