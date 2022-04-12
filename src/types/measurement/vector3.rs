use core::ops::{Add, AddAssign, Div, Mul, Sub};

use integer_sqrt::IntegerSquareRoot;
#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::{base::Scalar, ClosedAdd, ClosedDiv, ClosedMul, ClosedSub};
use serde::ser::SerializeSeq;

pub const X: usize = 0;
pub const Y: usize = 1;
pub const Z: usize = 2;

use super::vector::Vector;

pub trait Coordinate {}

#[derive(Copy, Clone, Debug, Default)]
pub struct Frame;

impl Coordinate for Frame {}

#[derive(Copy, Clone, Debug, Default)]
pub struct ENU;

impl Coordinate for ENU {}

#[derive(Copy, Clone, Debug)]
pub struct Vector3<T, U, C> {
    pub raw: nalgebra::Vector3<T>,
    unit: U,
    coordinate: C,
}

impl<T: Scalar + Default, U: Default, C: Default> Default for Vector3<T, U, C> {
    fn default() -> Self {
        Self { raw: Default::default(), unit: U::default(), coordinate: C::default() }
    }
}

impl<T, U, C> Vector3<T, U, C> {
    pub fn new(x: T, y: T, z: T, unit: U, coordinate: C) -> Self {
        Self { raw: nalgebra::Vector3::new(x, y, z), unit, coordinate }
    }

    pub fn from(raw: nalgebra::Vector3<T>, unit: U, coordinate: C) -> Self {
        Self { raw, unit, coordinate }
    }
}

impl<T: Copy + Default, U: Copy, C: Copy> Vector3<T, U, C> {
    pub fn x(&self) -> Vector<T, U> {
        Vector::new(self.raw[X], self.unit)
    }

    pub fn y(&self) -> Vector<T, U> {
        Vector::new(self.raw[Y], self.unit)
    }

    pub fn z(&self) -> Vector<T, U> {
        Vector::new(self.raw[Z], self.unit)
    }
}

impl<T: Copy + Default, U: Copy, C: Copy> Vector3<T, U, C> {
    pub fn t<V>(&self, convert: impl Fn(T) -> V + Copy) -> Vector3<V, U, C>
    where
        V: Copy + Default + PartialEq + serde::Serialize,
    {
        let (x, y, z) = (self.raw[0], self.raw[1], self.raw[2]);
        Vector3 {
            raw: nalgebra::Vector3::new(convert(x), convert(y), convert(z)),
            unit: self.unit,
            coordinate: self.coordinate,
        }
    }
}

impl<V, F: Copy + Default + Into<V>, C: Copy> Vector3<V, F, C>
where
    V: Mul<Output = V> + Div<Output = V> + Copy + Default + Scalar + ClosedMul + ClosedDiv,
{
    pub fn u<T: Copy + Default + Into<V>>(&self, unit: T) -> Vector3<V, T, C> {
        let from: V = F::default().into();
        let to: V = T::default().into();
        Vector3 { raw: self.raw * from / to, unit, coordinate: self.coordinate }
    }
}

impl<T, U, C> Into<nalgebra::Vector3<T>> for Vector3<T, U, C> {
    fn into(self) -> nalgebra::Vector3<T> {
        self.raw
    }
}

impl<T: Copy, U: Copy, C> Into<(T, T, T)> for Vector3<T, U, C> {
    fn into(self) -> (T, T, T) {
        (self.raw[X], self.raw[Y], self.raw[Z])
    }
}

impl<T: Copy, U: Copy, C> Into<[T; 3]> for Vector3<T, U, C> {
    fn into(self) -> [T; 3] {
        [self.raw[X], self.raw[Y], self.raw[Z]]
    }
}

impl<U: Copy + Default, C> Vector3<i32, U, C> {
    pub fn scalar(&self) -> Vector<u32, U> {
        let (x, y, z) = (self.raw[X], self.raw[Y], self.raw[Z]);
        Vector::new((x * x + y * y + z * z).integer_sqrt() as u32, U::default())
    }
}

impl<U: Copy + Default, C> Vector3<f32, U, C> {
    pub fn scalar(&self) -> Vector<f32, U> {
        let (x, y, z) = (self.raw[X], self.raw[Y], self.raw[Z]);
        Vector::new((x * x + y * y + z * z).sqrt(), U::default())
    }
}

impl<U: Copy + Default> Vector3<f32, U, ENU> {
    pub fn azimuth(&self) -> u16 {
        let theta = ((self.raw[X]).atan2(self.raw[Y]).to_degrees()) as i16;
        (if theta >= 0 { theta % 360 } else { 360 + theta }) as u16
    }
}

impl<T, U: Copy, C: Copy> Add for Vector3<T, U, C>
where
    T: Copy + Default + Add<Output = T> + Scalar + ClosedAdd,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self { raw: self.raw + rhs.raw, unit: self.unit, coordinate: self.coordinate }
    }
}

impl<T, U: Copy, C: Copy> AddAssign for Vector3<T, U, C>
where
    T: Copy + Default + Add<Output = T> + Scalar + ClosedAdd,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = Self { raw: self.raw + rhs.raw, unit: self.unit, coordinate: self.coordinate }
    }
}

impl<T, U: Copy, C: Copy> Sub for Vector3<T, U, C>
where
    T: Copy + Default + Sub<Output = T> + Scalar + ClosedSub,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self { raw: self.raw - rhs.raw, unit: self.unit, coordinate: self.coordinate }
    }
}

impl<T, U: Copy, C: Copy> Mul<T> for Vector3<T, U, C>
where
    T: Copy + Default + Mul<Output = T> + Scalar + ClosedMul,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self {
        Self { raw: self.raw * rhs, unit: self.unit, coordinate: self.coordinate }
    }
}

impl<T, U: Copy, C: Copy> Div<T> for Vector3<T, U, C>
where
    T: Copy + Default + Div<Output = T> + Scalar + ClosedDiv,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self {
        Self { raw: self.raw / rhs, unit: self.unit, coordinate: self.coordinate }
    }
}

impl<T: Copy + serde::Serialize, U: Copy, C> serde::Serialize for Vector3<T, U, C> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(3))?;
        for v in self.raw.iter() {
            seq.serialize_element(v)?;
        }
        seq.end()
    }
}

impl<'a, T, U: Default, C: Default> serde::Deserialize<'a> for Vector3<T, U, C>
where
    T: Copy + Scalar + serde::Deserialize<'a>,
{
    fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let array = <[T; 3]>::deserialize(deserializer)?;
        Ok(Self { raw: array.into(), unit: U::default(), coordinate: C::default() })
    }
}
