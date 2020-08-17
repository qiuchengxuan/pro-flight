use integer_sqrt::IntegerSquareRoot;
#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::Vector3;

pub mod battery;
pub mod distance;
pub mod euler;
pub mod velocity;

pub use distance::{Distance, DistanceUnit};
pub use euler::DEGREE_PER_DAG;
pub use velocity::Velocity;

pub type Temperature = i16;
pub type Altitude = Distance<i32>;

pub type Heading = u16;
pub type Course = u16;

#[derive(Copy, Clone)]
pub enum HeadingOrCourse {
    Heading(Heading),
    Course(Course),
}

impl Default for HeadingOrCourse {
    fn default() -> Self {
        Self::Course(0)
    }
}

impl Into<Heading> for HeadingOrCourse {
    fn into(self) -> Heading {
        match self {
            Self::Heading(h) => h,
            Self::Course(c) => c,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Value)]
pub struct Axes {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl core::ops::Add for Axes {
    type Output = Axes;

    fn add(self, other: Axes) -> Self {
        Self { x: (self.x + other.x), y: (self.y + other.y), z: (self.z + other.z) }
    }
}

impl core::ops::Sub<&Axes> for Axes {
    type Output = Axes;

    fn sub(self, other: &Axes) -> Self {
        Self { x: (self.x - other.x), y: (self.y - other.y), z: (self.z - other.z) }
    }
}

impl core::ops::Div<i32> for Axes {
    type Output = Axes;

    fn div(self, div: i32) -> Self {
        Self { x: self.x / div, y: self.y / div, z: self.z / div }
    }
}

impl core::ops::Mul<&Axes> for Axes {
    type Output = Axes;

    fn mul(self, other: &Axes) -> Self {
        Self { x: self.x * other.x, y: self.y * other.y, z: self.z * other.z }
    }
}

impl PartialOrd for Axes {
    fn partial_cmp(self: &Self, other: &Self) -> Option<core::cmp::Ordering> {
        if self.x > other.x || self.y > other.y || self.z > other.z {
            Some(core::cmp::Ordering::Greater)
        } else {
            Some(core::cmp::Ordering::Less)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Value)]
pub struct Measurement {
    pub axes: Axes,
    pub sensitive: i32,
}

impl Measurement {
    pub fn calibrated(self, axes: &Axes) -> Self {
        Self { axes: self.axes - axes, sensitive: self.sensitive }
    }
}

impl PartialOrd for Measurement {
    fn partial_cmp(self: &Self, other: &Self) -> Option<core::cmp::Ordering> {
        self.axes.partial_cmp(&other.axes)
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

#[derive(Debug, Default, Copy, Clone)]
pub struct Acceleration(pub Measurement);

impl sval::value::Value for Acceleration {
    fn stream(&self, stream: &mut sval::value::Stream) -> sval::value::Result {
        self.0.stream(stream)
    }
}

impl Acceleration {
    pub fn calibrated(self, zero: &Axes, gain: &Axes) -> Self {
        let axes = (self.0.axes - zero) * gain / self.0.sensitive;
        return Self(Measurement { axes, sensitive: self.0.sensitive });
    }
}

impl Acceleration {
    pub fn g_force(&self) -> u8 {
        let axes = self.0.axes;
        let (x, y, z) = (axes.x, axes.y, axes.z);
        let square_sum = x * x + y * y + z * z;
        if square_sum > 0 {
            let g_force = square_sum.integer_sqrt();
            (g_force * 10 / self.0.sensitive) as u8
        } else {
            0
        }
    }
}

pub type Gyro = Measurement;

#[derive(Copy, Clone, Default)]
pub struct Pressure(pub u32); // unit of Pa

impl Into<Altitude> for Pressure {
    fn into(self) -> Altitude {
        Distance(((1013_25 - self.0 as isize) * 82 / 10) as i32)
    }
}
