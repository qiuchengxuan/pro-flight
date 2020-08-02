use alloc::rc::Rc;

use ahrs::{Ahrs, Mahony};
use nalgebra::{Quaternion, UnitQuaternion, Vector3};

use crate::config::Accelerometer as Config;
use crate::datastructures::data_source::overwriting::{OverwritingData, OverwritingDataSource};
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{DataSource, DataWriter};
use crate::datastructures::measurement::{Acceleration, Axes, Gyro, DEGREE_PER_DAG};
use crate::datastructures::schedule::{Hertz, Schedulable};

pub struct IMU<A, G> {
    accelerometer: A,
    gyroscope: G,
    accelerometer_bias: Axes,
    calibration_loop: u16,

    ahrs: Mahony<f32>,
    gyro_bias: Axes,
    counter: usize,
    calibrated: bool,
    acceleration: Rc<OverwritingData<Acceleration>>,
    quaternion: Rc<OverwritingData<UnitQuaternion<f32>>>,
    gyro: Rc<SingularData<Gyro>>,
}

impl<A, G> IMU<A, G>
where
    A: DataSource<Acceleration>,
    G: DataSource<Gyro>,
{
    pub fn new(accelerometer: A, gyroscope: G, sample_rate: u16, config: &Config) -> Self {
        let size = accelerometer.capacity();
        let unit = UnitQuaternion::new_normalize(Quaternion::<f32>::new(0.0, 0.0, 0.0, 0.0));
        Self {
            accelerometer,
            gyroscope,
            accelerometer_bias: config.bias.into(),
            calibration_loop: 200,
            acceleration: Rc::new(OverwritingData::sized(size)),
            quaternion: Rc::new(OverwritingData::new(vec![unit; size])),
            gyro: Rc::new(SingularData::default()),

            ahrs: Mahony::new(1.0 / sample_rate as f32, 0.5, 0.0),
            gyro_bias: Default::default(),
            counter: 0,
            calibrated: false,
        }
    }

    pub fn set_calibration_loop(&mut self, value: u16) {
        self.calibration_loop = value;
    }

    pub fn as_accelerometer(&self) -> impl DataSource<Acceleration> {
        OverwritingDataSource::new(&self.acceleration)
    }

    pub fn as_imu(&self) -> impl DataSource<UnitQuaternion<f32>> {
        OverwritingDataSource::new(&self.quaternion)
    }

    pub fn as_gyroscope(&self) -> impl DataSource<Gyro> {
        SingularDataSource::new(&self.gyro)
    }

    pub fn update_imu(&mut self, raw: (Acceleration, Gyro)) {
        let acceleration = raw.0.calibrated(&self.accelerometer_bias);
        let gyro = raw.1.calibrated(&self.gyro_bias);
        self.acceleration.write(acceleration);
        self.gyro.write(gyro);

        let mut gyro: Vector3<f32> = gyro.into();
        gyro = gyro / DEGREE_PER_DAG;
        match self.ahrs.update_imu(&gyro, &(acceleration.0.into())) {
            Ok(&quaternion) => self.quaternion.write(UnitQuaternion::new_normalize(quaternion)),
            Err(_) => (),
        }
    }
}

impl<A: DataSource<Acceleration>, G: DataSource<Gyro>> Schedulable for IMU<A, G> {
    fn rate(&self) -> Hertz {
        50
    }

    fn schedule(&mut self) -> bool {
        while let Some(gyro) = self.gyroscope.read() {
            if let Some(acceleration) = self.accelerometer.read() {
                if !self.calibrated {
                    self.gyro_bias = self.gyro_bias.average(&gyro.axes);
                    self.calibrated = self.counter >= self.calibration_loop as usize;
                    self.counter += 1;
                } else {
                    self.update_imu((acceleration, gyro))
                }
            }
        }
        true
    }
}
