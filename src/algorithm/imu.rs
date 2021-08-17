use nalgebra::{UnitQuaternion, Vector3};

use crate::algorithm::mahony::{MagnetismOrHeading, Mahony};
use crate::config::imu::IMU as Config;
use crate::datastructures::measurement::euler::DEGREE_PER_DAG;
use crate::datastructures::measurement::{Acceleration, Axes, Gain, Gyro, Heading, Magnetism};

#[derive(PartialEq)]
pub enum Calibration {
    Calibrating,
    Validating,
    Calibrated,
}

pub struct IMU {
    ahrs: Mahony,
    accel_bias: Axes,
    accel_gain: Gain,
    gyro_bias: Axes,
    magnetometer_bias: Axes,
    magnetometer_gain: Gain,
    calibration_loop: usize,
    counter: usize,
    calibration: Calibration,
    acceleration: Vector3<f32>,
}

impl IMU {
    pub fn new(sample_rate: usize, config: &Config) -> Self {
        let (kp, ki) = (config.mahony.kp.into(), config.mahony.ki.into());
        Self {
            ahrs: Mahony::new(sample_rate as f32, kp, ki, config.magnetometer.declination.into()),
            accel_bias: config.accelerometer.bias.into(),
            accel_gain: config.accelerometer.gain,
            gyro_bias: Default::default(),
            magnetometer_bias: config.magnetometer.bias.into(),
            magnetometer_gain: config.magnetometer.gain,
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

    pub fn update_imu(
        &mut self,
        accel: &Acceleration,
        gyro: &Gyro,
        magnetism: Option<Magnetism>,
        heading: Option<Heading>,
    ) -> bool {
        if self.calibration != Calibration::Calibrated {
            self.calibrate(gyro);
            return false;
        }

        let acceleration = Acceleration(accel.0.zero(&self.accel_bias).gain(&self.accel_gain));
        let raw_gyro = gyro.zero(&self.gyro_bias);

        let acceleration: Vector3<f32> = acceleration.0.into();
        let mut gyro: Vector3<f32> = raw_gyro.into();
        gyro = gyro / DEGREE_PER_DAG;

        let heading = if let Some(mag) = magnetism {
            let m = mag.zero(&self.magnetometer_bias).gain(&self.magnetometer_gain).into();
            Some(MagnetismOrHeading::Magnetism(m))
        } else {
            heading.map(|h| MagnetismOrHeading::Heading(h.into()))
        };

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
