use alloc::boxed::Box;
use alloc::rc::Rc;

use ahrs::{Ahrs, Mahony};
#[allow(unused_imports)] // false warning
use micromath::F32Ext;
use nalgebra::{Quaternion, UnitQuaternion, Vector3};

use crate::components::schedule::{Hertz, Schedulable};
use crate::config;
use crate::datastructures::data_source::overwriting::{OverwritingData, OverwritingDataSource};
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{DataSource, DataWriter};
use crate::datastructures::measurement::{
    Acceleration, Axes, Gyro, Heading, HeadingOrCourse, DEGREE_PER_DAG,
};

pub struct IMU<A, G> {
    accelerometer: A,
    gyroscope: G,

    heading: Option<Box<dyn DataSource<HeadingOrCourse>>>,

    ahrs: Mahony<f32>,
    accelerometer_bias: Axes,
    accelerometer_gain: Axes,
    gyro_bias: Axes,
    calibration_loop: u16,
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
    pub fn new(accelerometer: A, gyroscope: G, sample_rate: u16) -> Self {
        let size = accelerometer.capacity();
        let unit = UnitQuaternion::new_normalize(Quaternion::<f32>::new(1.0, 0.0, 0.0, 0.0));
        let config = &config::get().accelerometer;
        Self {
            accelerometer,
            gyroscope,

            heading: None,

            ahrs: Mahony::new(1.0 / sample_rate as f32, 0.5, 0.0),
            accelerometer_bias: config.bias.into(),
            accelerometer_gain: config.gain.into(),
            gyro_bias: Default::default(),
            calibration_loop: 50,
            counter: 0,
            calibrated: false,
            acceleration: Rc::new(OverwritingData::sized(size)),
            quaternion: Rc::new(OverwritingData::new(vec![unit; size])),
            gyro: Rc::new(SingularData::default()),
        }
    }

    pub fn set_heading(&mut self, heading: Box<dyn DataSource<HeadingOrCourse>>) {
        self.heading = Some(heading);
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

    fn heading_as_magnitism(&self, heading: Heading) -> Option<Vector3<f32>> {
        let unit = UnitQuaternion::new_normalize(self.ahrs.quat);
        let forward = Vector3::new(1.0, 0.0, 0.0);
        let mut rotate_vector = unit.inverse_transform_vector(&forward);
        rotate_vector[2] = 0.0; // Rotate around z axis only
        if rotate_vector.norm_squared() > 0.01 {
            let heading = heading as f32 / DEGREE_PER_DAG;
            let vector = Vector3::new(heading.cos(), heading.sin(), 0.0);
            let vector = rotate_vector.normalize().cross(&vector);
            Some(unit.transform_vector(&vector))
        } else {
            None
        }
    }

    pub fn update_imu(&mut self, accel: &Acceleration, gyro: &Gyro, mag: Option<Vector3<f32>>) {
        let acceleration = accel.calibrated(&self.accelerometer_bias, &self.accelerometer_gain);
        let gyro = gyro.calibrated(&self.gyro_bias);
        self.acceleration.write(acceleration);
        self.gyro.write(gyro);

        let mut gyro: Vector3<f32> = gyro.into();
        gyro = gyro / DEGREE_PER_DAG;

        let result = if let Some(magnetism) = mag {
            self.ahrs.update(&gyro, &(acceleration.0.into()), &magnetism)
        } else {
            self.ahrs.update_imu(&gyro, &(acceleration.0.into()))
        };

        match result {
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
        if !self.calibrated {
            while let Some(gyro) = self.gyroscope.read() {
                self.gyro_bias = (self.gyro_bias + gyro.axes) / 2;
            }
            self.accelerometer.read_last_unchecked();
            self.calibrated = self.counter >= self.calibration_loop as usize;
            self.counter += 1;
            return true;
        }
        while let Some(gyro) = self.gyroscope.read() {
            let acceleration = self.accelerometer.read().unwrap();
            #[rustfmt::skip]
            let option = self.heading.as_mut().map(|h| h.read()).flatten()
                .map(|h| self.heading_as_magnitism(h.into())).flatten();
            self.update_imu(&acceleration, &gyro, option);
        }
        true
    }
}
