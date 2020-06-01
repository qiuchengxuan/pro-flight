use nalgebra::Vector3;

#[derive(Copy, Clone, Debug)]
pub struct Measurement {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub sensitive: f32,
}

impl Into<(f32, f32, f32)> for Measurement {
    fn into(self) -> (f32, f32, f32) {
        (
            self.x as f32 / self.sensitive,
            self.y as f32 / self.sensitive,
            self.z as f32 / self.sensitive,
        )
    }
}

impl Into<Vector3<f32>> for Measurement {
    fn into(self) -> Vector3<f32> {
        Vector3::new(
            self.x as f32 / self.sensitive,
            self.y as f32 / self.sensitive,
            self.z as f32 / self.sensitive,
        )
    }
}

pub type Acceleration = Measurement;
pub type Gyro = Measurement;

pub type Temperature<T> = T;
