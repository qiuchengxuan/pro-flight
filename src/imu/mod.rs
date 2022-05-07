pub mod out;

use fugit::NanosDurationU64 as Duration;
use nalgebra::Vector3;

use crate::{
    algorithm::mahony::{MagnetismOrHeading, Mahony},
    config, datastore,
    types::{
        measurement::{
            euler::{Euler, DEGREE_PER_DAG},
            unit::DEGs,
            Acceleration, Frame, Gyro,
        },
        sensor::{Bias, Gain, Readout},
    },
};

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

pub struct IMU {
    interval: Duration,
    ahrs: Mahony,
    calibration: Calibration,
}

impl IMU {
    pub fn new(sample_rate: usize) -> Self {
        let config = config::get().imu;
        let interval = Duration::micros(1000_000 / sample_rate as u64);
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
            interval,
            ahrs: Mahony::new(sample_rate as f32, kp, ki, config.magnetometer.declination.into()),
            calibration,
        }
    }

    /// gyro x, y, z means spin around x, y and z axis, clock-wise is positive
    pub fn update(&mut self, acceleration: Readout, gyro: Readout) {
        if self.calibration.status != CalibrationStatus::Calibrated {
            self.calibration.calibrate(gyro);
            return;
        }
        let ds = datastore::acquire();
        let gnss = ds.read_gnss_within(self.interval);
        let heading = gnss.map(|g| g.fixed.map(|f| f.heading)).flatten().flatten();
        let magnetism = ds.read_magnetism_within(self.interval);

        let calib = &self.calibration.accelerometer;
        let raw_acceleration = acceleration.zero(&calib.bias).gain(&calib.gain);
        let raw_gyro = gyro - self.calibration.gyroscope_bias;

        let acceleration: Vector3<f32> = raw_acceleration.into();
        let gyro: Vector3<f32> = raw_gyro.into();

        let heading = if let Some(mag) = magnetism {
            let calib = &self.calibration.magnetometer;
            let m = mag.zero(&calib.bias).gain(&calib.gain).into();
            Some(MagnetismOrHeading::Magnetism(m))
        } else {
            heading.map(|h| MagnetismOrHeading::Heading(h.0.into()))
        };

        if !self.ahrs.update(gyro / DEGREE_PER_DAG, acceleration, heading) {
            return;
        }
        let quaternion = self.ahrs.quaternion();
        let acceleration = Acceleration::new(acceleration, Frame);
        let gyro = Gyro::new(gyro, DEGs);
        let attitude = Euler::from(quaternion) * DEGREE_PER_DAG;
        ds.write_imu(out::IMU { acceleration, gyro, quaternion, attitude })
    }

    /// Testing only
    pub fn skip_calibration(&mut self) {
        self.calibration.status = CalibrationStatus::Calibrated;
    }
}
