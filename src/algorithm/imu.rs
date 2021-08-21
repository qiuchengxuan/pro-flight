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

#[derive(Default)]
struct Sensor {
    bias: Axes,
    gain: Gain,
}

pub struct IMU {
    ahrs: Mahony,
    accelerometer: Sensor,
    gyroscope: Sensor,
    magnetometer: Sensor,
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
            accelerometer: Sensor {
                bias: config.accelerometer.bias.into(),
                gain: config.accelerometer.gain,
            },
            gyroscope: Default::default(),
            magnetometer: Sensor {
                bias: config.magnetometer.bias.into(),
                gain: config.magnetometer.gain,
            },
            calibration_loop: sample_rate,
            counter: 0,
            calibration: Calibration::Calibrating,
            acceleration: Default::default(),
        }
    }

    fn calibrate(&mut self, gyro: &Gyro) {
        match self.calibration {
            Calibration::Calibrating => {
                self.gyroscope.bias = (self.gyroscope.bias + gyro.axes) / 2;
                self.counter += 1;
                if self.counter >= self.calibration_loop {
                    self.calibration = Calibration::Validating;
                    self.counter = 0;
                }
            }
            Calibration::Validating => {
                let delta = gyro.axes - self.gyroscope.bias;
                let (x, y, z) = (delta.x, delta.y, delta.z);
                let sensitive = gyro.sensitive as i32;
                if x.abs() > sensitive || y.abs() > sensitive || z.abs() > sensitive {
                    warn!("IMU calibration invalid, restarting...");
                    self.calibration = Calibration::Calibrating;
                    self.counter = 0;
                } else if self.counter >= self.calibration_loop {
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
        acceleration: &Acceleration,
        gyro: &Gyro,
        magnetism: Option<Magnetism>,
        heading: Option<Heading>,
    ) -> bool {
        if self.calibration != Calibration::Calibrated {
            self.calibrate(gyro);
            return false;
        }

        let accel = &self.accelerometer;
        let acceleration = Acceleration(acceleration.0.zero(&accel.bias).gain(&accel.gain));
        let raw_gyro = gyro.zero(&self.gyroscope.bias);

        let acceleration: Vector3<f32> = acceleration.0.into();
        let mut gyro: Vector3<f32> = raw_gyro.into();
        gyro = gyro / DEGREE_PER_DAG;

        let heading = if let Some(mag) = magnetism {
            let m = mag.zero(&self.magnetometer.bias).gain(&self.magnetometer.gain).into();
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
