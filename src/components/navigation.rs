use nalgebra::UnitQuaternion;

use crate::alloc;
use crate::datastructures::coordinate::{Displacement, Position};
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{DataSource, DataWriter};
use crate::datastructures::measurement::Acceleration;
use crate::datastructures::schedule::Schedulable;
use crate::datastructures::waypoint::{Steerpoint, Waypoint};
use crate::math::runge_kutta4;

const CURRENT: usize = 0;
const HOME: usize = 0;
const MAX_WAYPOINT: usize = 32;

pub struct Navigation<'a, IMU, A> {
    imu: IMU,
    accelerometer: A,
    gnss: Option<&'a mut dyn DataSource<Position>>,
    waypoints: [Waypoint; MAX_WAYPOINT],
    displacements: [Displacement; MAX_WAYPOINT],
    output: &'static SingularData<(Position, Steerpoint)>,
    next_waypoint: u8,
    max_waypoint: u8,
    interval: f32,
    time: f32,
}

impl<'a, IMU, A> Navigation<'a, IMU, A>
where
    IMU: DataSource<UnitQuaternion<f32>>,
    A: DataSource<Acceleration>,
{
    pub fn new(imu: IMU, accelerometer: A, interval: f32) -> Self {
        let output = SingularData::<(Position, Steerpoint)>::default();
        Self {
            imu,
            accelerometer,
            gnss: None,
            waypoints: [Waypoint::default(); MAX_WAYPOINT],
            displacements: [Displacement::default(); MAX_WAYPOINT],
            output: alloc::into_static(output, false).unwrap(),
            next_waypoint: HOME as u8,
            max_waypoint: 1,
            interval,
            time: 0.0,
        }
    }

    pub fn as_data_source(&self) -> impl DataSource<(Position, Steerpoint)> {
        SingularDataSource::new(&self.output)
    }

    pub fn set_gnss(&mut self, gnss: &'a mut dyn DataSource<Position>) {
        self.gnss = Some(gnss)
    }

    pub fn next_waypoint(&mut self) {
        self.next_waypoint = (self.next_waypoint + 1) % self.max_waypoint;
    }

    pub fn update_waypoint(&mut self, index: usize, waypoint: Waypoint) {
        if index >= MAX_WAYPOINT || index != self.max_waypoint as usize + 1 {
            return;
        }
        self.waypoints[index] = waypoint;
        self.max_waypoint += 1;
        if index != HOME {
            self.displacements[index] = waypoint.position - self.waypoints[HOME].position;
        }
    }

    fn update_from_imu(&mut self) {
        if let Some(unit_quaternion) = self.imu.read() {
            if let Some(acceleration) = self.accelerometer.read() {
                let vector = unit_quaternion.transform_vector(&acceleration.0.into()) * 9.8;
                let (ax, ay, az) = (vector[0], vector[1], vector[2]);
                let mut current = self.displacements[CURRENT];
                current.x = runge_kutta4(|_, dt| ax * dt, current.x, self.time, self.interval);
                current.y = runge_kutta4(|_, dt| ay * dt, current.y, self.time, self.interval);
                current.z = runge_kutta4(|_, dt| az * dt, current.z, self.time, self.interval);
                self.time += self.interval;
                self.displacements[CURRENT] = current;
                let waypoint = self.waypoints[self.next_waypoint as usize];
                let steerpoint = Steerpoint { index: self.next_waypoint, waypoint };
                let position = self.waypoints[HOME].position + self.displacements[CURRENT];
                self.output.write((position, steerpoint));
            }
        }
    }

    pub fn update_from_gnss(&mut self) {
        if let Some(position) = self.gnss.as_mut().map(|gnss| gnss.read_last()).flatten() {
            self.displacements[CURRENT] = self.waypoints[HOME].position - position;
            self.time = 0.0;
        }
    }
}

impl<'a, IMU, A> Schedulable for Navigation<'a, IMU, A>
where
    IMU: DataSource<UnitQuaternion<f32>>,
    A: DataSource<Acceleration>,
{
    fn schedule(&mut self) {
        self.update_from_imu();
        self.update_from_gnss();
    }
}
