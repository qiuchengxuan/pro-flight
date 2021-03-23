#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::{UnitQuaternion, Vector3};

use crate::algorithm::mahony::{MagnetismOrHeading, Mahony};
use crate::config;
use crate::datastructures::measurement::euler::DEGREE_PER_DAG;
use crate::datastructures::measurement::{Acceleration, Axes, Course, Gyro, Heading, Magnetism};
use crate::sync::{AgingDataReader, DataReader};

#[derive(PartialEq)]
pub enum Calibration {
    Calibrating,
    Validating,
    Calibrated,
}

pub struct IMU<M, H, C> {
    magnetometer: M,
    heading: H,
    course: C,
    aging: usize,
    ahrs: Mahony,
    accel_bias: Axes,
    accel_gain: Axes,
    gyro_bias: Axes,
    magnetometer_bias: Axes,
    magnetometer_gain: Axes,
    calibration_loop: u16,
    counter: usize,
    calibration: Calibration,
    acceleration: Vector3<f32>,
}

impl<M, H, C> IMU<M, H, C>
where
    M: DataReader<Magnetism>,
    H: AgingDataReader<Heading>,
    C: AgingDataReader<Course>,
{
    pub fn new(magnetometer: M, heading: H, course: C, sample_rate: u16, aging: usize) -> Self {
        let config = &config::get().imu;
        let (kp, ki) = (config.mahony.kp.into(), config.mahony.ki.into());
        Self {
            magnetometer,
            heading,
            course,
            aging,

            ahrs: Mahony::new(sample_rate as f32, kp, ki, config.magnetometer.declination.into()),
            accel_bias: config.accelerometer.bias.into(),
            accel_gain: config.accelerometer.gain.into(),
            gyro_bias: Default::default(),
            magnetometer_bias: config.magnetometer.bias.into(),
            magnetometer_gain: config.magnetometer.gain.into(),
            calibration_loop: sample_rate,
            counter: 0,
            calibration: Calibration::Calibrating,
            acceleration: Default::default(),
        }
    }

    fn calibrate(&mut self, gyro: &Gyro) {
        match self.calibration {
            Calibration::Calibrating => {
                self.gyro_bias = (self.gyro_bias + gyro.axes) / 2;
                self.counter += 1;
                if self.counter >= self.calibration_loop as usize {
                    self.calibration = Calibration::Validating;
                    self.counter = 0;
                }
            }
            Calibration::Validating => {
                let delta = gyro.axes - self.gyro_bias;
                let (x, y, z) = (delta.x, delta.y, delta.z);
                let sensitive = gyro.sensitive;
                if x.abs() > sensitive || y.abs() > sensitive || z.abs() > sensitive {
                    warn!("IMU calibration invalid, restarting...");
                    self.calibration = Calibration::Calibrating;
                    self.counter = 0;
                } else if self.counter >= self.calibration_loop as usize {
                    info!("IMU calibration finished");
                    self.calibration = Calibration::Calibrated;
                } else {
                    self.counter += 1;
                }
            }
            _ => (),
        }
    }

    pub fn update_imu(&mut self, accel: &Acceleration, gyro: &Gyro) -> bool {
        if self.calibration != Calibration::Calibrated {
            self.calibrate(gyro);
            return false;
        }

        let heading = if let Some(mag) = self.magnetometer.get_last() {
            let (bias, gain) = (&self.magnetometer_bias, &self.magnetometer_gain);
            Some(MagnetismOrHeading::Magnetism(mag.zero(bias).gain(gain).into()))
        } else {
            let aging = self.aging;
            let option = self.heading.get_aging_last(aging).and(self.course.get_aging_last(aging));
            option.map(|h| MagnetismOrHeading::Heading(h.into()))
        };

        let acceleration = Acceleration(accel.0.zero(&self.accel_bias).gain(&self.accel_gain));
        let raw_gyro = gyro.zero(&self.gyro_bias);

        let acceleration: Vector3<f32> = acceleration.0.into();
        let mut gyro: Vector3<f32> = raw_gyro.into();
        gyro = gyro / DEGREE_PER_DAG;

        if self.ahrs.update(&gyro, &acceleration, heading) {
            self.acceleration = self.ahrs.quaternion().transform_vector(&acceleration);
            return true;
        }
        false
    }

    pub fn quaternion(&self) -> UnitQuaternion<f32> {
        self.ahrs.quaternion()
    }

    pub fn acceleration(&self) -> Vector3<f32> {
        self.acceleration
    }
}
