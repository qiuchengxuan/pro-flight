use alloc::boxed::Box;
use alloc::rc::Rc;

#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::{Quaternion, UnitQuaternion, Vector3};

use crate::algorithm::mahony::{MagnetismOrHeading as Heading, Mahony};
use crate::components::schedule::{Rate, Schedulable};
use crate::config;
use crate::datastructures::data_source::overwriting::{OverwritingData, OverwritingDataSource};
use crate::datastructures::data_source::{
    AgingStaticData, DataWriter, OptionData, StaticData, WithCapacity,
};
use crate::datastructures::measurement::euler::DEGREE_PER_DAG;
use crate::datastructures::measurement::{Acceleration, Axes, Gyro, HeadingOrCourse, Magnetism};

pub struct IMU<A, G> {
    accelerometer: A,
    gyroscope: G,

    magnetometer: Option<Box<dyn StaticData<Magnetism>>>,
    gnss: Option<Box<dyn AgingStaticData<HeadingOrCourse>>>,

    ahrs: Mahony,
    accel_bias: Axes,
    accel_gain: Axes,
    gyro_bias: Axes,
    magnetometer_bias: Axes,
    magnetometer_gain: Axes,
    calibration_loop: u16,
    counter: usize,
    calibrated: bool,
    acceleration: Rc<OverwritingData<Vector3<f32>>>,
    quaternion: Rc<OverwritingData<UnitQuaternion<f32>>>,
}

impl<A: OptionData<Acceleration> + WithCapacity, G: OptionData<Gyro>> IMU<A, G> {
    pub fn new(accelerometer: A, gyroscope: G, sample_rate: u16) -> Self {
        let size = accelerometer.capacity();
        let acceleration = Vector3::<f32>::new(0.0, 0.0, 0.0);
        let unit = UnitQuaternion::new_normalize(Quaternion::<f32>::new(1.0, 0.0, 0.0, 0.0));
        let config = &config::get().imu;
        let (kp, ki) = (config.mahony.kp.into(), config.mahony.ki.into());
        Self {
            accelerometer,
            gyroscope,

            magnetometer: None,
            gnss: None,

            ahrs: Mahony::new(sample_rate as f32, kp, ki, config.magnetometer.declination.into()),
            accel_bias: config.accelerometer.bias.into(),
            accel_gain: config.accelerometer.gain.into(),
            gyro_bias: Default::default(),
            magnetometer_bias: config.magnetometer.bias.into(),
            magnetometer_gain: config.magnetometer.gain.into(),
            calibration_loop: 50,
            counter: 0,
            calibrated: false,
            acceleration: Rc::new(OverwritingData::new(vec![acceleration; size])),
            quaternion: Rc::new(OverwritingData::new(vec![unit; size])),
        }
    }

    pub fn set_magnetometer(&mut self, magnetometer: Box<dyn StaticData<Magnetism>>) {
        self.magnetometer = Some(magnetometer);
    }

    pub fn set_gnss(&mut self, gnss: Box<dyn AgingStaticData<HeadingOrCourse>>) {
        self.gnss = Some(gnss);
    }

    pub fn set_calibration_loop(&mut self, value: u16) {
        self.calibration_loop = value;
    }

    pub fn as_accelerometer(&self) -> OverwritingDataSource<Vector3<f32>> {
        OverwritingDataSource::new(&self.acceleration)
    }

    pub fn reader(&self) -> OverwritingDataSource<UnitQuaternion<f32>> {
        OverwritingDataSource::new(&self.quaternion)
    }

    fn calibrate(&mut self) -> bool {
        let mut min = Axes::MAX;
        let mut max = Axes::MIN;
        let mut bias = self.gyro_bias;
        let mut sensitive = 0;
        while let Some(gyro) = self.gyroscope.read() {
            self.accelerometer.read();
            min.x = core::cmp::min(min.x, gyro.axes.x);
            min.y = core::cmp::min(min.y, gyro.axes.y);
            min.z = core::cmp::min(min.z, gyro.axes.z);
            max.x = core::cmp::max(max.x, gyro.axes.x);
            max.y = core::cmp::max(max.y, gyro.axes.y);
            max.z = core::cmp::max(max.z, gyro.axes.z);
            bias = (bias + gyro.axes) / 2;
            sensitive = gyro.sensitive;
        }
        if max.x - min.x > sensitive || max.y - min.y > sensitive || max.z - min.z > sensitive {
            return true;
        }
        self.gyro_bias = (self.gyro_bias + bias) / 2;
        self.counter += 1;
        self.calibrated = self.counter >= self.calibration_loop as usize;
        true
    }

    pub fn update_imu(&mut self, accel: &Acceleration, gyro: &Gyro, heading: Option<Heading>) {
        let acceleration = Acceleration(accel.0.zero(&self.accel_bias).gain(&self.accel_gain));
        let raw_gyro = gyro.zero(&self.gyro_bias);

        let acceleration: Vector3<f32> = acceleration.0.into();
        let mut gyro: Vector3<f32> = raw_gyro.into();
        gyro = gyro / DEGREE_PER_DAG;

        if let Some(quaternion) = self.ahrs.update(&gyro, &acceleration, heading) {
            let acceleration = quaternion.transform_vector(&acceleration);
            self.acceleration.write(acceleration);
            self.quaternion.write(quaternion);
        }
    }
}

impl<A: OptionData<Acceleration> + WithCapacity, G: OptionData<Gyro>> Schedulable for IMU<A, G> {
    fn rate(&self) -> Rate {
        50
    }

    fn schedule(&mut self) -> bool {
        if !self.calibrated {
            return self.calibrate();
        }
        let rate = self.rate();
        let heading = if let Some(mag) = self.magnetometer.as_mut() {
            let (bias, gain) = (&self.magnetometer_bias, &self.magnetometer_gain);
            Some(Heading::Magnetism(mag.read().zero(bias).gain(gain).into()))
        } else if let Some(gnss) = self.gnss.as_mut() {
            gnss.read(rate).map(|h| Heading::Heading(h.or_course().into()))
        } else {
            None
        };
        while let Some(gyro) = self.gyroscope.read() {
            let acceleration = self.accelerometer.read().unwrap();
            self.update_imu(&acceleration, &gyro, heading);
        }
        true
    }
}
