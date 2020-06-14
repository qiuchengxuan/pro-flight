use ahrs::{Ahrs, Mahony};
use integer_sqrt::IntegerSquareRoot;
use nalgebra::Vector3;

use crate::config::Accelerometer;
use crate::datastructures::measurement::{quaternion_to_euler, Euler, DEGREE_PER_DAG};
use crate::datastructures::ring_buffer::RingBufferReader;
use crate::hal::sensors::{Acceleration, Axes, Gyro};

pub struct IMU<'a> {
    reader: RingBufferReader<'a, (Acceleration, Gyro)>,
    accelerometer_bias: Axes,
    calibration_loop: u16,

    ahrs: Mahony<f32>,
    gyro_bias: Axes,
    acceleration: Acceleration,
    gyro: Gyro,
    counter: usize,
    calibrated: bool,
}

impl<'a> IMU<'a> {
    pub fn new(
        reader: RingBufferReader<'a, (Acceleration, Gyro)>,
        sample_rate: u16,
        config: &Accelerometer,
        calibration_loop: u16,
    ) -> Self {
        Self {
            reader,
            accelerometer_bias: config.bias.into(),
            calibration_loop,

            ahrs: Mahony::new(1.0 / sample_rate as f32, 0.5, 0.0),
            gyro_bias: Default::default(),
            acceleration: Default::default(),
            gyro: Default::default(),
            counter: 0,
            calibrated: false,
        }
    }

    pub fn update(&mut self) {
        while let Some((acceleration, gyro)) = self.reader.read() {
            self.counter += 1;
            if !self.calibrated {
                self.gyro = gyro;
                self.gyro_bias = self.gyro_bias.average(&gyro.axes);
                self.calibrated = self.counter >= self.calibration_loop as usize;
            }

            self.acceleration = acceleration;
            self.gyro = gyro;
            self.acceleration.axes.calibrated(&self.accelerometer_bias);
            self.gyro.axes.calibrated(&self.gyro_bias);

            let mut gyro: Vector3<f32> = self.gyro.into();
            gyro = gyro / DEGREE_PER_DAG;
            self.ahrs.update_imu(&gyro, &(acceleration.into())).ok();
        }
    }

    pub fn get_zyx_euler(&self) -> Euler {
        quaternion_to_euler(&self.ahrs.quat)
    }

    pub fn g_force(&self) -> u8 {
        let acceleration = self.acceleration;
        let axes = acceleration.axes;
        let (x, y, z) = (axes.x as i32, axes.y as i32, axes.z as i32);
        let g_force = (x * x + y * y + z * z).integer_sqrt();
        let g_force = (g_force / acceleration.sensitive * 100) as u16;
        ((g_force + 5) / 10) as u8
    }
}
