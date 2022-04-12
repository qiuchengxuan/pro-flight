use nalgebra::Vector3;

use crate::{
    algorithm::mahony::{MagnetismOrHeading, Mahony},
    config::imu::IMU as Config,
    types::{
        measurement::{euler::DEGREE_PER_DAG, unit::DEGs, Acceleration, Gyro, Heading, ENU},
        sensor::{Bias, Gain, Readout},
    },
};

pub mod out;

#[derive(PartialEq)]
pub enum CalibrationStatus {
    Calibrating,
    Validating,
    Calibrated,
}

#[derive(Default)]
struct Sensor {
    bias: Bias,
    gain: Gain,
}

struct Calibration {
    accelerometer: Sensor,
    gyroscope_bias: Readout,
    magnetometer: Sensor,
    initial: usize,
    remain: usize,
    status: CalibrationStatus,
}

impl Calibration {
    fn calibrate(&mut self, gyro: Readout) {
        match self.status {
            CalibrationStatus::Calibrating => {
                self.gyroscope_bias = (self.gyroscope_bias + gyro) / 2;
                self.remain -= 1;
                if self.remain == 0 {
                    self.remain = self.initial;
                    self.status = CalibrationStatus::Validating;
                }
            }
            CalibrationStatus::Validating => {
                let delta = gyro - self.gyroscope_bias;
                let (x, y, z) = (delta.axes.x, delta.axes.y, delta.axes.z);
                let sensitive = gyro.sensitive as i32;
                if x.abs() > sensitive || y.abs() > sensitive || z.abs() > sensitive {
                    warn!("IMU status invalid, restarting...");
                    self.status = CalibrationStatus::Calibrating;
                    self.remain = self.initial;
                } else if self.remain == 0 {
                    info!("IMU status finished");
                    self.status = CalibrationStatus::Calibrated;
                } else {
                    self.remain -= 1
                }
            }
            _ => (),
        }
    }
}

pub struct Input {
    pub acceleration: Readout,
    pub gyro: Readout,
    pub magnetism: Option<Readout>,
    pub heading: Option<Heading>,
}

pub struct IMU {
    ahrs: Mahony,
    calibration: Calibration,
    acceleration: Vector3<f32>,
}

impl IMU {
    pub fn new(sample_rate: usize, config: &Config) -> Self {
        let (kp, ki) = (config.mahony.kp.into(), config.mahony.ki.into());
        let calibration = Calibration {
            accelerometer: Sensor {
                bias: config.accelerometer.bias,
                gain: config.accelerometer.gain,
            },
            gyroscope_bias: Default::default(),
            magnetometer: Sensor { bias: config.magnetometer.bias, gain: config.magnetometer.gain },
            initial: sample_rate,
            remain: sample_rate,
            status: CalibrationStatus::Calibrating,
        };
        Self {
            ahrs: Mahony::new(sample_rate as f32, kp, ki, config.magnetometer.declination.into()),
            calibration,
            acceleration: Default::default(),
        }
    }

    pub fn update_imu(&mut self, input: Input) -> Option<out::IMU> {
        if self.calibration.status != CalibrationStatus::Calibrated {
            self.calibration.calibrate(input.gyro);
            return None;
        }

        let calib = &self.calibration.accelerometer;
        let raw_acceleration = input.acceleration.zero(&calib.bias).gain(&calib.gain);
        let raw_gyro = input.gyro - self.calibration.gyroscope_bias;

        let acceleration: Vector3<f32> = raw_acceleration.into();
        let mut gyro: Vector3<f32> = raw_gyro.into();
        gyro = gyro / DEGREE_PER_DAG;

        let heading = if let Some(mag) = input.magnetism {
            let calib = &self.calibration.magnetometer;
            let m = mag.zero(&calib.bias).gain(&calib.gain).into();
            Some(MagnetismOrHeading::Magnetism(m))
        } else {
            input.heading.map(|h| MagnetismOrHeading::Heading(h.0.into()))
        };

        if !self.ahrs.update(&gyro, &acceleration, heading) {
            return None;
        }
        self.acceleration = self.ahrs.quaternion().transform_vector(&acceleration);
        let quaternion = self.ahrs.quaternion();
        let acceleration = Acceleration::new(self.acceleration, ENU);
        let gyro = Gyro::new(gyro, DEGs);
        Some(out::IMU { acceleration, gyro, quaternion, attitude: quaternion.into() })
    }

    /// Testing only
    pub fn skip_calibration(&mut self) {
        self.calibration.status = CalibrationStatus::Calibrated;
    }
}
