use super::rotation::Rotation;

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize)]
pub struct Axes {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Axes {
    pub const MAX: Axes = Axes { x: i32::MAX, y: i32::MAX, z: i32::MAX };
    pub const MIN: Axes = Axes { x: i32::MIN, y: i32::MIN, z: i32::MIN };

    pub fn rotate(self, rotation: Rotation) -> Self {
        let (x, y, z) = (self.x, self.y, self.z);
        let (x, y, z) = match rotation {
            Rotation::NoRotation => (x, y, z),
            Rotation::Degree90 => (y, x, z),
            Rotation::Degree180 => (-x, -y, z),
            Rotation::Degree270 => (-y, x, z),
        };
        Self { x, y, z }
    }
}

impl core::ops::Add for Axes {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self { x: (self.x + other.x), y: (self.y + other.y), z: (self.z + other.z) }
    }
}

impl core::ops::Sub<Self> for Axes {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self { x: (self.x - other.x), y: (self.y - other.y), z: (self.z - other.z) }
    }
}

impl core::ops::Sub<&Self> for Axes {
    type Output = Self;

    fn sub(self, other: &Self) -> Self {
        Self { x: (self.x - other.x), y: (self.y - other.y), z: (self.z - other.z) }
    }
}

impl core::ops::Div<i32> for Axes {
    type Output = Self;

    fn div(self, div: i32) -> Self {
        Self { x: self.x / div, y: self.y / div, z: self.z / div }
    }
}

impl core::ops::Mul<&Self> for Axes {
    type Output = Self;

    fn mul(self, other: &Self) -> Self {
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
