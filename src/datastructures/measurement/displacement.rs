use core::ops::{Add, AddAssign, Div, Mul, Sub};

use integer_sqrt::IntegerSquareRoot as SquareRoot;
#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::Vector3;

use super::distance::Distance;
use super::unit::Meter;

#[derive(Copy, Clone, Default, Debug)]
pub struct DistanceVector<T, U> {
    pub x: Distance<T, U>,
    pub y: Distance<T, U>,
    pub z: Distance<T, U>,
}

impl<T: Copy + Default + PartialEq + sval::Value, U: Copy> sval::Value for DistanceVector<T, U> {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        stream.map_begin(Some(3))?;
        stream.map_key("x")?;
        stream.map_value(self.x)?;
        stream.map_key("y")?;
        stream.map_value(self.y)?;
        stream.map_key("z")?;
        stream.map_value(self.z)?;
        stream.map_end()
    }
}

impl<T: Copy + Default, U: Copy> DistanceVector<T, U> {
    pub fn new(x: T, y: T, z: T, unit: U) -> Self {
        Self { x: Distance::new(x, unit), y: Distance::new(y, unit), z: Distance::new(z, unit) }
    }

    pub fn convert<V>(&self, convert: impl Fn(T) -> V + Copy) -> DistanceVector<V, U>
    where
        V: Copy + Default + PartialEq + sval::Value,
    {
        DistanceVector {
            x: self.x.convert(convert),
            y: self.y.convert(convert),
            z: self.z.convert(convert),
        }
    }
}

impl<V, F: Copy + Default + Into<V>> DistanceVector<V, F>
where
    V: Mul<Output = V> + Div<Output = V> + Copy + Default + PartialEq + sval::Value,
{
    pub fn to_unit<T: Copy + Default + Into<V>>(&self, unit: T) -> DistanceVector<V, T> {
        DistanceVector { x: self.x.to_unit(unit), y: self.y.to_unit(unit), z: self.z.to_unit(unit) }
    }
}

impl Into<Vector3<f32>> for DistanceVector<f32, Meter> {
    fn into(self) -> Vector3<f32> {
        Vector3::new(self.x.value(), self.y.value(), self.z.value())
    }
}

impl From<Vector3<f32>> for DistanceVector<f32, Meter> {
    fn from(vector: Vector3<f32>) -> Self {
        Self::new(vector[0], vector[1], vector[2], Meter)
    }
}

impl From<(f32, f32, f32)> for DistanceVector<f32, Meter> {
    fn from(tuple: (f32, f32, f32)) -> Self {
        Self {
            x: Distance::new(tuple.0, Meter),
            y: Distance::new(tuple.1, Meter),
            z: Distance::new(tuple.2, Meter),
        }
    }
}

impl<U: Copy> Into<(f32, f32, f32)> for DistanceVector<f32, U> {
    fn into(self) -> (f32, f32, f32) {
        (self.x.value(), self.y.value(), self.z.value())
    }
}

impl<U: Copy + Default> DistanceVector<i32, U> {
    pub fn distance(&self) -> Distance<u32, U> {
        let (x, y, z) = (self.x.value(), self.y.value(), self.z.value());
        Distance::new((x * x + y * y + z * z).integer_sqrt() as u32, U::default())
    }
}

impl<U: Copy + Default> DistanceVector<f32, U> {
    pub fn azimuth(&self) -> u16 {
        let theta = ((self.x.value()).atan2(self.y.value()).to_degrees()) as i16;
        (if theta >= 0 { theta % 360 } else { 360 + theta }) as u16
    }

    pub fn distance(&self) -> Distance<f32, U> {
        let (x, y, z) = (self.x.value(), self.y.value(), self.z.value());
        Distance::new((x * x + y * y + z * z).sqrt(), U::default())
    }
}

impl<T, U: Copy> Add for DistanceVector<T, U>
where
    T: Copy + Default + PartialEq + Add<Output = T>,
{
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }
}

impl<T, U: Copy> AddAssign for DistanceVector<T, U>
where
    T: Copy + Default + PartialEq + Add<Output = T>,
{
    fn add_assign(&mut self, other: Self) {
        *self = Self { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }
}

impl<T, U: Copy> Sub for DistanceVector<T, U>
where
    T: Copy + Default + PartialEq + Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
}

impl<T, U: Copy> Mul<T> for DistanceVector<T, U>
where
    T: Copy + Default + PartialEq + Mul<Output = T>,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self {
        Self { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}

impl<T, U: Copy> Div<T> for DistanceVector<T, U>
where
    T: Copy + Default + PartialEq + Div<Output = T>,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self {
        Self { x: self.x / rhs, y: self.y / rhs, z: self.z / rhs }
    }
}
