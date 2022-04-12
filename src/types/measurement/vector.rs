use core::{
    fmt, marker,
    ops::{Add, AddAssign, Div, Mul, Sub},
};

#[derive(Copy, Clone, Default, Debug)]
pub struct Vector<T, U> {
    pub raw: T,
    unit: marker::PhantomData<U>,
}

impl<T: Copy + Default, U: Copy> Vector<T, U> {
    pub fn new(value: T, _: U) -> Self {
        Self { raw: value, unit: core::marker::PhantomData }
    }

    #[inline]
    pub fn t<V: Copy + Default>(self, convert: impl Fn(T) -> V) -> Vector<V, U> {
        Vector { raw: convert(self.raw), unit: marker::PhantomData }
    }
}

impl<T: fmt::Display, U: fmt::Display + Default> fmt::Display for Vector<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.raw, U::default())
    }
}

impl<T: From<u8> + Copy + Default + PartialEq, U: Copy> Vector<T, U> {
    pub fn is_zero(&self) -> bool {
        self.raw == T::from(0)
    }
}

impl<T: PartialEq + Copy + Default, U: Copy> PartialEq for Vector<T, U> {
    fn eq(&self, rhs: &Self) -> bool {
        self.raw == rhs.raw
    }
}

impl<T: Add<Output = T> + Copy + Default + PartialEq, U: Copy> Add for Vector<T, U> {
    type Output = Vector<T::Output, U>;

    fn add(self, other: Self) -> Self::Output {
        Self { raw: self.raw + other.raw, unit: marker::PhantomData }
    }
}

impl<T: AddAssign + Copy + Default + PartialEq, U: Copy> AddAssign for Vector<T, U> {
    fn add_assign(&mut self, other: Self) {
        self.raw += other.raw
    }
}

impl<T: Sub<Output = T> + Copy + Default + PartialEq, U: Copy> Sub for Vector<T, U> {
    type Output = Vector<T::Output, U>;

    fn sub(self, other: Self) -> Self::Output {
        Self { raw: self.raw - other.raw, unit: marker::PhantomData }
    }
}

impl<T: Mul<Output = T> + Copy + Default + PartialEq, U: Copy> Mul<T> for Vector<T, U> {
    type Output = Vector<T::Output, U>;

    fn mul(self, t: T) -> Self::Output {
        Self { raw: self.raw * t, unit: marker::PhantomData }
    }
}

impl<T: Div<Output = T> + Copy + Default + PartialEq, U: Copy> Div<T> for Vector<T, U> {
    type Output = Vector<T::Output, U>;

    fn div(self, t: T) -> Self::Output {
        Self { raw: self.raw / t, unit: marker::PhantomData }
    }
}

impl<V, F: Copy + Default + Into<V>> Vector<V, F>
where
    V: Mul<Output = V> + Div<Output = V> + Copy + Default,
{
    pub fn u<T: Copy + Default + Into<V>>(self, _: T) -> Vector<V, T> {
        let from: V = F::default().into();
        let to: V = T::default().into();
        Vector { raw: self.raw * from / to, unit: marker::PhantomData }
    }
}

impl<T: serde::Serialize, U: Copy> serde::Serialize for Vector<T, U> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.raw.serialize(serializer)
    }
}

impl<'a, T: serde::Deserialize<'a>, U> serde::Deserialize<'a> for Vector<T, U> {
    fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self { raw: T::deserialize(deserializer)?, unit: marker::PhantomData })
    }
}
